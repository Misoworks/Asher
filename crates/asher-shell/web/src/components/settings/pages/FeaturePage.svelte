<script lang="ts">
  import { featureDefinition } from "../../../settings/feature_pages";
  import type { SettingsPage, SettingsPatch, SettingsSnapshot } from "../../../settings/model";

  let { page, settings, apply } = $props<{
    page: SettingsPage;
    settings: SettingsSnapshot;
    apply: (patch: SettingsPatch) => Promise<void>;
  }>();

  const definition = $derived(featureDefinition(page, settings));
</script>

{#each definition.groups as group (group.title)}
  <section class="settings-section">
    <h2>{group.title}</h2>
    <div class="settings-group">
      {#each group.rows as row (row.label)}
        <div class="settings-row feature-row">
          <div>
            <span>{row.label}</span>
            {#if row.detail}
              <p>{row.detail}</p>
            {/if}
          </div>
          {#if row.control}
            {@const control = row.control}
            {#if control.type === "switch"}
              <button
                type="button"
                class="settings-switch"
                class:is-on={control.active}
                aria-label={row.label}
                aria-pressed={control.active}
                onclick={() => apply(control.patch)}
              >
                <span></span>
              </button>
            {:else}
              <div class="choice-row compact">
                {#each control.options as option (option.value ?? "off")}
                  <button
                    type="button"
                    class:is-active={control.selected === option.value}
                    onclick={() => apply(option.patch)}
                  >
                    {option.label}
                  </button>
                {/each}
              </div>
            {/if}
          {:else}
            <strong>{row.value}</strong>
          {/if}
        </div>
      {/each}
    </div>
  </section>
{/each}
