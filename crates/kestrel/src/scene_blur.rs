use crate::layers::{BlurLayer, LayerMaterial, LayerRenderTarget};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Blit, Frame, Offscreen, Renderer, TextureFilter,
            element::{Id, Kind, texture::TextureRenderElement},
            gles::{
                GlesError, GlesRenderer, GlesTarget, GlesTexProgram, GlesTexture, Uniform,
                UniformName, UniformType, UniformValue,
            },
        },
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Buffer, Logical, Physical, Point, Rectangle, Size, Transform},
};
use std::time::Instant;

const BLUR_SHADER: &str = r#"#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision mediump float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
uniform vec2 texel;
uniform vec2 target_size;
uniform float radius;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

float hash(vec2 value) {
    return fract(sin(dot(value, vec2(127.1, 311.7))) * 43758.5453123) - 0.5;
}

float roundedCoverage(vec2 pixel) {
    if (radius <= 0.0) {
        return 1.0;
    }

    vec2 inner_min = vec2(radius);
    vec2 inner_max = target_size - vec2(radius);
    vec2 closest = clamp(pixel, inner_min, inner_max);
    vec2 delta = pixel - closest;
    float distance = length(delta);
    return 1.0 - smoothstep(radius - 1.0, radius + 1.0, distance);
}

void main() {
    vec4 color = texture2D(tex, v_coords) * 0.18;
    color += texture2D(tex, v_coords + texel * vec2(1.384615, 0.0)) * 0.16;
    color += texture2D(tex, v_coords - texel * vec2(1.384615, 0.0)) * 0.16;
    color += texture2D(tex, v_coords + texel * vec2(0.0, 1.384615)) * 0.16;
    color += texture2D(tex, v_coords - texel * vec2(0.0, 1.384615)) * 0.16;
    color += texture2D(tex, v_coords + texel * vec2(1.0, 1.0)) * 0.095;
    color += texture2D(tex, v_coords + texel * vec2(-1.0, 1.0)) * 0.095;
    color += texture2D(tex, v_coords + texel * vec2(1.0, -1.0)) * 0.095;
    color += texture2D(tex, v_coords + texel * vec2(-1.0, -1.0)) * 0.095;

    float luma = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    color.rgb = luma + (color.rgb - vec3(luma)) * 1.04;
    color.rgb += hash(gl_FragCoord.xy) * 0.0043;

    float coverage = roundedCoverage(v_coords * target_size);
    color.a = coverage * alpha;
    color.rgb *= color.a;

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = vec4(0.0, 0.2, 0.0, 0.2) + color * 0.8;
#endif

    gl_FragColor = color;
}
"#;

#[derive(Default)]
pub struct SceneBlurCache {
    entries: Vec<SceneBlurCacheEntry>,
    program: Option<GlesTexProgram>,
}

impl SceneBlurCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn retain_targets(&mut self, targets: &[LayerRenderTarget]) {
        for entry in &mut self.entries {
            if let Some(target) = targets
                .iter()
                .find(|target| target.size.w > 1 && target.size.h > 1 && entry.matches(target))
            {
                entry.location = target.location;
                entry.target_opacity = target.opacity;
            }
        }
        self.entries
            .retain(|entry| targets.iter().any(|target| entry.matches(target)));
    }

    pub fn is_animating(&self) -> bool {
        false
    }

    pub fn has_cached_elements(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn targets_need_capture(
        &self,
        output_size: Size<i32, Physical>,
        targets: &[LayerRenderTarget],
        damage: &[Rectangle<i32, Physical>],
    ) -> bool {
        targets.iter().any(|target| {
            let Some(rect) = clipped_target_rect(output_size, target) else {
                return false;
            };
            self.cached_entry(target)
                .is_none_or(|entry| entry.rect != rect || target_is_damaged(rect, damage))
        })
    }

    pub fn cached_elements(
        &self,
        renderer: &mut GlesRenderer,
        output_size: Size<i32, Physical>,
        _blur_layer: BlurLayer,
        targets: &[LayerRenderTarget],
    ) -> Result<Vec<BlurElement>, GlesError> {
        let now = Instant::now();
        let mut elements = Vec::new();
        for target in targets {
            let Some(rect) = clipped_target_rect(output_size, target) else {
                continue;
            };
            let Some(entry) = self.cached_entry(target) else {
                continue;
            };
            elements.push(render_element(
                renderer,
                rect,
                entry,
                entry.opacity(now, target.opacity),
            ));
        }

        Ok(elements)
    }

    fn program(&mut self, renderer: &mut GlesRenderer) -> Result<GlesTexProgram, GlesError> {
        if let Some(program) = &self.program {
            return Ok(program.clone());
        }

        let program = renderer.compile_custom_texture_shader(
            BLUR_SHADER,
            &[
                UniformName::new("texel", UniformType::_2f),
                UniformName::new("target_size", UniformType::_2f),
                UniformName::new("radius", UniformType::_1f),
            ],
        )?;
        self.program = Some(program.clone());
        Ok(program)
    }

    fn buffer_for_target(
        &mut self,
        renderer: &mut GlesRenderer,
        framebuffer: &GlesTarget<'_>,
        output_size: Size<i32, Physical>,
        target: &LayerRenderTarget,
        rect: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
    ) -> Result<(&SceneBlurCacheEntry, f32), GlesError> {
        let now = Instant::now();
        let cached = self.entries.iter().position(|entry| entry.matches(target));
        if let Some(index) = cached
            && !target_is_damaged(rect, damage)
            && self.entries[index].rect == rect
        {
            self.entries[index].location = target.location;
            self.entries[index].target_opacity = target.opacity;
            let opacity = self.entries[index].opacity(now, target.opacity);
            return Ok((&self.entries[index], opacity));
        }

        let program = self.program(renderer)?;
        let texture_size = blur_texture_size(rect.size);
        let (capture, blurred) = match cached {
            Some(index) if self.entries[index].texture_size == texture_size => {
                let entry = &mut self.entries[index];
                capture_target(
                    renderer,
                    framebuffer,
                    output_size,
                    rect,
                    texture_size,
                    &mut entry.capture,
                )?;
                render_blur_texture(
                    renderer,
                    &program,
                    target.material,
                    texture_size,
                    &entry.capture,
                    &mut entry.blurred,
                )?;
                entry.location = target.location;
                entry.target_opacity = target.opacity;
                let opacity = entry.opacity(now, target.opacity);
                return Ok((&self.entries[index], opacity));
            }
            _ => {
                let mut capture = renderer.create_buffer(
                    Fourcc::Abgr8888,
                    Size::<i32, Buffer>::from((texture_size.w, texture_size.h)),
                )?;
                let mut blurred = renderer.create_buffer(
                    Fourcc::Abgr8888,
                    Size::<i32, Buffer>::from((texture_size.w, texture_size.h)),
                )?;
                capture_target(
                    renderer,
                    framebuffer,
                    output_size,
                    rect,
                    texture_size,
                    &mut capture,
                )?;
                render_blur_texture(
                    renderer,
                    &program,
                    target.material,
                    texture_size,
                    &capture,
                    &mut blurred,
                )?;
                (capture, blurred)
            }
        };

        match cached {
            Some(index) => {
                self.entries[index] = SceneBlurCacheEntry {
                    id: self.entries[index].id.clone(),
                    surface: target.surface.clone(),
                    blur_layer: target.blur_layer,
                    rect,
                    location: target.location,
                    size: target.size,
                    material: target.material,
                    texture_size,
                    capture,
                    blurred,
                    target_opacity: target.opacity,
                };
            }
            None => self.entries.push(SceneBlurCacheEntry {
                id: Id::new(),
                surface: target.surface.clone(),
                blur_layer: target.blur_layer,
                rect,
                location: target.location,
                size: target.size,
                material: target.material,
                texture_size,
                capture,
                blurred,
                target_opacity: target.opacity,
            }),
        }

        let index = cached.unwrap_or(self.entries.len() - 1);
        let opacity = self.entries[index].opacity(now, target.opacity);
        Ok((&self.entries[index], opacity))
    }

    fn cached_entry(&self, target: &LayerRenderTarget) -> Option<&SceneBlurCacheEntry> {
        self.entries.iter().find(|entry| entry.matches(target))
    }
}

pub type BlurElement = TextureRenderElement<GlesTexture>;

pub fn capture_blur_elements(
    cache: &mut SceneBlurCache,
    renderer: &mut GlesRenderer,
    framebuffer: &GlesTarget<'_>,
    output_size: Size<i32, Physical>,
    targets: &[LayerRenderTarget],
    damage: &[Rectangle<i32, Physical>],
    enabled: bool,
) -> Result<Vec<BlurElement>, GlesError> {
    if !enabled {
        return Ok(Vec::new());
    }

    let mut elements = Vec::new();
    for target in targets {
        let Some(rect) = clipped_target_rect(output_size, target) else {
            continue;
        };
        let (entry, opacity) =
            cache.buffer_for_target(renderer, framebuffer, output_size, target, rect, damage)?;
        elements.push(render_element(renderer, rect, entry, opacity));
    }

    Ok(elements)
}

#[derive(Debug)]
struct SceneBlurCacheEntry {
    id: Id,
    surface: WlSurface,
    blur_layer: BlurLayer,
    rect: Rectangle<i32, Physical>,
    location: Point<i32, Logical>,
    size: Size<i32, Logical>,
    material: LayerMaterial,
    texture_size: Size<i32, Physical>,
    capture: GlesTexture,
    blurred: GlesTexture,
    target_opacity: f32,
}

impl SceneBlurCacheEntry {
    fn matches(&self, target: &LayerRenderTarget) -> bool {
        self.surface == target.surface
            && self.blur_layer == target.blur_layer
            && (self.blur_layer != BlurLayer::Window || self.location == target.location)
            && self.size == target.size
            && self.material == target.material
    }

    fn opacity(&self, _now: Instant, current_opacity: f32) -> f32 {
        current_opacity.clamp(0.0, 1.0)
    }
}

fn capture_target(
    renderer: &mut GlesRenderer,
    framebuffer: &GlesTarget<'_>,
    output_size: Size<i32, Physical>,
    rect: Rectangle<i32, Physical>,
    texture_size: Size<i32, Physical>,
    capture: &mut GlesTexture,
) -> Result<(), GlesError> {
    let source = Rectangle::<i32, Physical>::new(
        (rect.loc.x, output_size.h - rect.loc.y - rect.size.h).into(),
        rect.size,
    );
    let target = Rectangle::<i32, Physical>::from_size(texture_size);
    let mut target_framebuffer = renderer.bind(capture)?;
    renderer.blit(
        framebuffer,
        &mut target_framebuffer,
        source,
        target,
        TextureFilter::Linear,
    )
}

fn render_blur_texture(
    renderer: &mut GlesRenderer,
    program: &GlesTexProgram,
    material: LayerMaterial,
    texture_size: Size<i32, Physical>,
    capture: &GlesTexture,
    blurred: &mut GlesTexture,
) -> Result<(), GlesError> {
    let mut target = renderer.bind(blurred)?;
    let mut frame = renderer.render(&mut target, texture_size, Transform::Flipped180)?;
    let full_damage = [Rectangle::<i32, Physical>::from_size(texture_size)];
    frame.clear(
        smithay::backend::renderer::Color32F::new(0.0, 0.0, 0.0, 0.0),
        &full_damage,
    )?;
    frame.render_texture_from_to(
        capture,
        Rectangle::<f64, Buffer>::from_size(Size::<f64, Buffer>::from((
            texture_size.w as f64,
            texture_size.h as f64,
        ))),
        Rectangle::<i32, Physical>::from_size(texture_size),
        &full_damage,
        &[],
        Transform::Normal,
        1.0,
        Some(program),
        &blur_uniforms(texture_size, material),
    )?;
    let _ = frame.finish()?;
    Ok(())
}

fn render_element(
    renderer: &GlesRenderer,
    rect: Rectangle<i32, Physical>,
    entry: &SceneBlurCacheEntry,
    opacity: f32,
) -> BlurElement {
    TextureRenderElement::from_static_texture(
        entry.id.clone(),
        renderer.context_id(),
        Point::<f64, Physical>::from((rect.loc.x as f64, rect.loc.y as f64)),
        entry.blurred.clone(),
        1,
        Transform::Normal,
        Some(opacity.clamp(0.0, 1.0)),
        Some(Rectangle::<f64, Logical>::from_size(
            Size::<f64, Logical>::from((entry.texture_size.w as f64, entry.texture_size.h as f64)),
        )),
        Some(Size::<i32, Logical>::from((rect.size.w, rect.size.h))),
        None,
        Kind::Unspecified,
    )
}

fn blur_uniforms(
    texture_size: Size<i32, Physical>,
    material: LayerMaterial,
) -> [Uniform<'static>; 3] {
    [
        Uniform::new(
            "texel",
            UniformValue::_2f(
                1.0 / texture_size.w.max(1) as f32,
                1.0 / texture_size.h.max(1) as f32,
            ),
        ),
        Uniform::new(
            "target_size",
            UniformValue::_2f(texture_size.w as f32, texture_size.h as f32),
        ),
        Uniform::new(
            "radius",
            UniformValue::_1f(material_radius(material, texture_size)),
        ),
    ]
}

fn material_radius(material: LayerMaterial, texture_size: Size<i32, Physical>) -> f32 {
    match material {
        LayerMaterial::Rect => 0.0,
        LayerMaterial::RoundRect { radius } => {
            let scale = blur_downscale(texture_size.w, texture_size.h) as f32;
            radius as f32 / scale
        }
    }
}

fn clipped_target_rect(
    output_size: Size<i32, Physical>,
    target: &LayerRenderTarget,
) -> Option<Rectangle<i32, Physical>> {
    clipped_rect(output_size, target.location, target.size)
}

fn clipped_rect(
    output_size: Size<i32, Physical>,
    location: Point<i32, Logical>,
    size: Size<i32, Logical>,
) -> Option<Rectangle<i32, Physical>> {
    if size.w <= 1 || size.h <= 1 {
        return None;
    }

    let output = Rectangle::<i32, Physical>::from_size(output_size);
    Rectangle::<i32, Physical>::new((location.x, location.y).into(), (size.w, size.h).into())
        .intersection(output)
}

fn blur_texture_size(size: Size<i32, Physical>) -> Size<i32, Physical> {
    let scale = blur_downscale(size.w, size.h);
    Size::<i32, Physical>::from((
        div_ceil(size.w, scale).max(1),
        div_ceil(size.h, scale).max(1),
    ))
}

fn blur_downscale(width: i32, height: i32) -> i32 {
    let area = width.saturating_mul(height);
    if area >= 420_000 {
        12
    } else if area >= 120_000 {
        10
    } else {
        7
    }
}

fn div_ceil(value: i32, divisor: i32) -> i32 {
    (value + divisor - 1) / divisor
}

fn target_is_damaged(rect: Rectangle<i32, Physical>, damage: &[Rectangle<i32, Physical>]) -> bool {
    damage
        .iter()
        .any(|damage| damage.intersection(rect).is_some())
}
