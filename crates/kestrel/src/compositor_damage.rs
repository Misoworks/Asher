use crate::{
    damage::{
        DamageTracker, LayerGeometryTracker, blur_damage_elements, damage_area, damage_elements,
        expand_damage_for_blur_targets, merge_damage_rectangles,
    },
    layers::{self, LayerRenderTarget},
    render::LayerElement,
    submitted_damage::SubmittedDamageHistory,
    window_clip::RoundedWindowElement,
};
use smithay::{
    backend::renderer::{
        element::{memory::MemoryRenderBufferRenderElement, surface::WaylandSurfaceRenderElement},
        gles::GlesRenderer,
    },
    output::Output,
    utils::{Physical, Rectangle, Size},
};

type LayerSurfaceElement = LayerElement;
type WindowSurfaceElement = WaylandSurfaceRenderElement<GlesRenderer>;
type MemoryElement = MemoryRenderBufferRenderElement<GlesRenderer>;
type WindowElement = RoundedWindowElement<WindowSurfaceElement>;

pub struct CompositorDamagePlan {
    pub damage: Vec<Rectangle<i32, Physical>>,
    pub blur_damage: Vec<Rectangle<i32, Physical>>,
    pub damage_area: i32,
}

pub struct CompositorDamageContext<'a> {
    pub output_size: Size<i32, Physical>,
    pub output: &'a Output,
    pub buffer_age: usize,
    pub force_full_damage: bool,
    pub blur_enabled: bool,
    pub blur_animating: bool,
    pub window_effect_targets: &'a [LayerRenderTarget],
    pub top_targets: &'a [LayerRenderTarget],
    pub overlay_targets: &'a [LayerRenderTarget],
    pub background: Option<&'a MemoryElement>,
    pub background_layer: &'a [LayerSurfaceElement],
    pub bottom_layer: &'a [LayerSurfaceElement],
    pub windows: &'a [WindowElement],
    pub window_chrome: &'a [MemoryElement],
    pub top_layer: &'a [LayerSurfaceElement],
    pub overlay_layer: &'a [LayerSurfaceElement],
    pub loading: Option<&'a MemoryElement>,
    pub debug: Option<&'a MemoryElement>,
}

pub fn plan_compositor_damage(
    ctx: CompositorDamageContext<'_>,
    damage_tracker: &mut DamageTracker,
    blur_damage_tracker: &mut DamageTracker,
    layer_geometry: &mut LayerGeometryTracker,
    submitted_damage: &SubmittedDamageHistory,
) -> CompositorDamagePlan {
    let force_damage = ctx.force_full_damage || ctx.blur_animating;
    let damage_plan = {
        let elements = damage_elements(
            ctx.background,
            ctx.background_layer,
            ctx.bottom_layer,
            ctx.windows,
            ctx.window_chrome,
            ctx.top_layer,
            ctx.overlay_layer,
            ctx.loading,
            ctx.debug,
        );
        damage_tracker.plan(ctx.output_size, ctx.buffer_age, force_damage, &elements)
    };
    let blur_damage_plan = {
        let elements = blur_damage_elements(
            ctx.background,
            ctx.background_layer,
            ctx.bottom_layer,
            ctx.windows,
        );
        blur_damage_tracker.plan(ctx.output_size, ctx.buffer_age, force_damage, &elements)
    };
    let damage = if ctx.blur_enabled {
        expand_damage_for_blur_targets(
            ctx.output_size,
            &damage_plan.rectangles,
            &blur_damage_plan.rectangles,
            &[
                ctx.window_effect_targets,
                ctx.top_targets,
                ctx.overlay_targets,
            ],
        )
    } else {
        damage_plan.rectangles.clone()
    };
    let (damage, geometry_changed) = layer_geometry.expand_damage(
        ctx.output_size,
        &damage,
        &layers::layer_surface_rects(ctx.output),
    );
    let blur_damage = if ctx.blur_enabled {
        expand_damage_for_blur_targets(
            ctx.output_size,
            &blur_damage_plan.rectangles,
            &damage_plan.rectangles,
            &[
                ctx.window_effect_targets,
                ctx.top_targets,
                ctx.overlay_targets,
            ],
        )
    } else {
        blur_damage_plan.rectangles.clone()
    };
    let blur_damage = merge_damage_rectangles(
        Rectangle::<i32, Physical>::from_size(ctx.output_size),
        blur_damage
            .into_iter()
            .chain(damage.iter().copied())
            .collect(),
    );
    let force_geometry_damage = geometry_changed || ctx.blur_animating;
    let damage = submitted_damage.accumulate(ctx.output_size, &damage, ctx.buffer_age);
    let blur_damage = if force_geometry_damage && ctx.blur_enabled {
        vec![Rectangle::<i32, Physical>::from_size(ctx.output_size)]
    } else {
        submitted_damage.accumulate(ctx.output_size, &blur_damage, ctx.buffer_age)
    };

    CompositorDamagePlan {
        damage_area: damage_area(&damage),
        damage,
        blur_damage,
    }
}
