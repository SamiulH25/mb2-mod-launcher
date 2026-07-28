<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { AppState, ModuleState } from "./lib/types";
  import { scheduleStatusDismiss, cancelStatusDismiss } from "./lib/status";
  import {
    getVisibleRange,
    MOD_ROW_HEIGHT,
    VISIBLE_ROW_COUNT,
    VIRTUAL_LIST_THRESHOLD,
  } from "./lib/virtual-scroll";
  import CoatOfArms from "./lib/components/CoatOfArms.svelte";
  import CalradiaBanner from "./lib/components/CalradiaBanner.svelte";
  import ModCard from "./lib/components/ModCard.svelte";
  import LoadOrderTimeline from "./lib/components/LoadOrderTimeline.svelte";
  import QuickActionsPanel from "./lib/components/QuickActionsPanel.svelte";
  import OrnamentDivider from "./lib/components/OrnamentDivider.svelte";

  let appState: AppState | null = $state(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let statusMessage = $state<string | null>(null);
  let searchQuery = $state("");
  let dragIndex: number | null = $state(null);
  let dragOverIndex: number | null = $state(null);
  let expandedId: string | null = $state(null);
  let listScrollTop = $state(0);
  let listViewportHeight = $state(MOD_ROW_HEIGHT * VISIBLE_ROW_COUNT);
  let listBodyEl: HTMLDivElement | null = $state(null);
  let scrollRaf: number | null = null;

  const modules = $derived(appState?.modules ?? []);
  const normalizedQuery = $derived(searchQuery.trim().toLowerCase());

  const filteredModules = $derived.by(() => {
    if (!normalizedQuery) {
      return modules;
    }
    return modules.filter((m) => {
      const info = m.module.info;
      return (
        info.name.toLowerCase().includes(normalizedQuery) ||
        info.id.toLowerCase().includes(normalizedQuery)
      );
    });
  });

  const enabledCount = $derived(modules.filter((m) => m.enabled).length);
  const totalCount = $derived(modules.length);

  const enabledWaypoints = $derived.by(() => {
    const waypoints: { id: string; name: string }[] = [];
    for (const module of modules) {
      if (module.enabled) {
        waypoints.push({
          id: module.module.info.id,
          name: module.module.info.name,
        });
      }
    }
    return waypoints;
  });

  const useVirtualList = $derived(
    filteredModules.length > VIRTUAL_LIST_THRESHOLD && expandedId === null,
  );

  const virtualWindow = $derived.by(() => {
    if (!useVirtualList) {
      return { start: 0, end: filteredModules.length, offsetY: 0 };
    }
    const { start, end } = getVisibleRange(
      listScrollTop,
      listViewportHeight,
      filteredModules.length,
      MOD_ROW_HEIGHT,
    );
    return { start, end, offsetY: start * MOD_ROW_HEIGHT };
  });

  const visibleModules = $derived(
    useVirtualList
      ? filteredModules.slice(virtualWindow.start, virtualWindow.end)
      : filteredModules,
  );

  const moduleIndexById = $derived.by(() => {
    const map = new Map<string, number>();
    filteredModules.forEach((module, index) => {
      map.set(module.module.info.id, index);
    });
    return map;
  });

  const listSpacerHeight = $derived(filteredModules.length * MOD_ROW_HEIGHT);

  $effect(() => {
    if (!listBodyEl) return;

    const updateViewport = () => {
      listViewportHeight = listBodyEl?.clientHeight ?? listViewportHeight;
    };

    updateViewport();
    const observer = new ResizeObserver(updateViewport);
    observer.observe(listBodyEl);

    return () => observer.disconnect();
  });

  onMount(() => {
    return () => {
      if (scrollRaf !== null) {
        cancelAnimationFrame(scrollRaf);
      }
      cancelStatusDismiss();
    };
  });

  function showStatus(message: string) {
    statusMessage = message;
    scheduleStatusDismiss(() => {
      statusMessage = null;
    });
  }

  function showError(message: string) {
    error = message;
    scheduleStatusDismiss(() => {
      error = null;
    });
  }

  async function detectGame() {
    loading = true;
    error = null;
    statusMessage = null;
    cancelStatusDismiss();
    try {
      appState = await invoke<AppState>("detect_game");
      showStatus(`Found ${appState.modules.length} modules`);
    } catch (e) {
      showError(String(e));
      appState = null;
    } finally {
      loading = false;
    }
  }

  async function refreshModules() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      appState = await invoke<AppState>("refresh_modules");
      showStatus("Modules refreshed");
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  async function toggleModule(module: ModuleState) {
    if (!appState) return;
    try {
      appState = await invoke<AppState>("toggle_module", {
        moduleId: module.module.info.id,
        enabled: !module.enabled,
      });
    } catch (e) {
      showError(String(e));
    }
  }

  async function enableAllModules() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      appState = await invoke<AppState>("set_all_modules_enabled", { enabled: true });
      showStatus(`Enlisted all ${appState.modules.length} modules`);
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  async function autoSort() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      appState = await invoke<AppState>("auto_sort_modules");
      showStatus("Load order sorted");
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  async function saveLoadOrder() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      await invoke("save_load_order");
      showStatus("Saved to LauncherData.xml");
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  async function launchGame() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      await invoke("launch_game");
      showStatus("Saved load order and launching Bannerlord via Steam…");
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  async function unblockDlls() {
    if (!appState) return;
    loading = true;
    error = null;
    cancelStatusDismiss();
    try {
      const results = await invoke<Array<{ files_unblocked: number }>>("unblock_dlls");
      const total = results.reduce((sum, r) => sum + r.files_unblocked, 0);
      showStatus(`Unblocked ${total} DLL file(s)`);
    } catch (e) {
      showError(String(e));
    } finally {
      loading = false;
    }
  }

  function onListScroll(event: Event) {
    const target = event.currentTarget as HTMLDivElement;
    if (scrollRaf !== null) return;
    scrollRaf = requestAnimationFrame(() => {
      listScrollTop = target.scrollTop;
      scrollRaf = null;
    });
  }

  function moduleIdFromEvent(event: Event): string | null {
    const row = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-module-id]");
    return row?.dataset.moduleId ?? null;
  }

  function onListDragStart(event: DragEvent) {
    const moduleId = moduleIdFromEvent(event);
    if (!moduleId) return;
    dragIndex = moduleIndexById.get(moduleId) ?? null;
  }

  function onListDragOver(event: DragEvent) {
    event.preventDefault();
    const moduleId = moduleIdFromEvent(event);
    if (!moduleId) return;
    const index = moduleIndexById.get(moduleId);
    if (index === undefined || dragOverIndex === index) return;
    dragOverIndex = index;
  }

  function onListDragLeave(event: DragEvent) {
    const related = event.relatedTarget as Node | null;
    if (related && listBodyEl?.contains(related)) return;
    dragOverIndex = null;
  }

  async function onListDrop(event: DragEvent) {
    const targetModuleId = moduleIdFromEvent(event);
    dragOverIndex = null;
    if (!appState || dragIndex === null || !targetModuleId) {
      dragIndex = null;
      return;
    }

    const movedModule = filteredModules[dragIndex];
    if (!movedModule || movedModule.module.info.id === targetModuleId) {
      dragIndex = null;
      return;
    }

    const ids = appState.modules.map((m) => m.module.info.id);
    const from = ids.indexOf(movedModule.module.info.id);
    const to = ids.indexOf(targetModuleId);
    dragIndex = null;

    if (from < 0 || to < 0 || from === to) {
      return;
    }

    const [moved] = ids.splice(from, 1);
    ids.splice(to, 0, moved);

    try {
      appState = await invoke<AppState>("reorder_modules", { moduleIds: ids });
    } catch (e) {
      showError(String(e));
    }
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function dismissStatus() {
    cancelStatusDismiss();
    statusMessage = null;
    error = null;
  }
</script>

<div class="app-shell">
  <CalradiaBanner
    subtitle={appState
      ? `${enabledCount} of ${totalCount} modules enlisted`
      : "Bannerlord mod management for Linux"}
    {enabledCount}
    {totalCount}
    {loading}
    hasGame={!!appState}
    onDetect={detectGame}
    onRefresh={refreshModules}
    onLaunch={launchGame}
  />

  <div class="main-column">
    {#if error}
      <div class="toast error" role="alert">
        <span>{error}</span>
        <button class="toast-x" onclick={dismissStatus} aria-label="Dismiss">×</button>
      </div>
    {/if}

    {#if statusMessage}
      <div class="toast success" role="status">
        <span>{statusMessage}</span>
        <button class="toast-x" onclick={dismissStatus} aria-label="Dismiss">×</button>
      </div>
    {/if}

    {#if appState}
      <div class="workspace">
        <section class="roster-column">
          <div class="roster-head panel">
            <div class="roster-title-row">
              <h2>Module Roster</h2>
              <button
                class="seal roster-enable-all"
                type="button"
                onclick={enableAllModules}
                disabled={loading}
              >
                Enable All
              </button>
            </div>
            <span class="roster-hint">
              {filteredModules.length} modules · showing {VISIBLE_ROW_COUNT} at a time · drag to reorder
            </span>
          </div>

          <div class="list-head panel">
            <span>Rank</span>
            <span>Module</span>
            <span></span>
            <span>Ver.</span>
            <span>Origin</span>
            <span></span>
            <span></span>
          </div>

          <div
            class="list-body panel scroll-region"
            role="list"
            aria-label="Installed modules"
            bind:this={listBodyEl}
            onscroll={onListScroll}
            ondragstart={onListDragStart}
            ondragover={onListDragOver}
            ondragleave={onListDragLeave}
            ondrop={onListDrop}
          >
            {#if filteredModules.length === 0}
              <div class="list-empty">
                <span class="empty-mark" aria-hidden="true">⚑</span>
                <p>No modules match your search.</p>
              </div>
            {:else if useVirtualList}
              <div class="virtual-spacer" style:height="{listSpacerHeight}px">
                <div class="virtual-window" style:transform="translateY({virtualWindow.offsetY}px)">
                  {#each visibleModules as module (module.module.info.id)}
                    {@const index = moduleIndexById.get(module.module.info.id) ?? 0}
                    <ModCard
                      {module}
                      {index}
                      expanded={false}
                      dragging={dragIndex === index}
                      dragOver={dragOverIndex === index}
                      ontoggle={() => toggleModule(module)}
                      onexpand={() => toggleExpand(module.module.info.id)}
                    />
                  {/each}
                </div>
              </div>
            {:else}
              {#each visibleModules as module (module.module.info.id)}
                {@const index = moduleIndexById.get(module.module.info.id) ?? 0}
                <ModCard
                  {module}
                  {index}
                  expanded={expandedId === module.module.info.id}
                  dragging={dragIndex === index}
                  dragOver={dragOverIndex === index}
                  ontoggle={() => toggleModule(module)}
                  onexpand={() => toggleExpand(module.module.info.id)}
                />
              {/each}
            {/if}
          </div>
        </section>

        <QuickActionsPanel
          bind:searchQuery
          gamePath={appState.paths.game_root}
          configPath={appState.paths.launcher_data}
          {loading}
          warnings={appState.warnings}
          onLaunch={launchGame}
          onAutoSort={autoSort}
          onUnblockDlls={unblockDlls}
          onSave={saveLoadOrder}
          onRefresh={refreshModules}
        />
      </div>

      <LoadOrderTimeline waypoints={enabledWaypoints} />
    {:else}
      <section class="welcome panel-inset panel-heraldic">
        <CoatOfArms size={80} />
        <OrnamentDivider label="Enlist" />
        <h2>Rally Your Mods</h2>
        <p>
          Detect your Steam install to muster modules from <code>SubModule.xml</code>,
          arrange your campaign load order, and deploy to <code>LauncherData.xml</code>.
        </p>
        <ul class="welcome-features">
          <li>
            <span class="feat-icon" aria-hidden="true">⚔</span>
            <span>Workshop &amp; game module scanning</span>
          </li>
          <li>
            <span class="feat-icon" aria-hidden="true">⟳</span>
            <span>Dependency-aware auto sort</span>
          </li>
          <li>
            <span class="feat-icon" aria-hidden="true">⚿</span>
            <span>Proton DLL unblocking</span>
          </li>
        </ul>
        <button class="seal seal-primary welcome-cta" onclick={detectGame} disabled={loading}>
          {loading ? "Scouting…" : "Detect Game"}
        </button>
      </section>
    {/if}
  </div>
</div>

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    padding: 0.6rem 0.75rem;
    gap: 0.5rem;
  }

  .main-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    gap: 0.45rem;
    min-height: 0;
  }

  .toast {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.55rem 0.75rem;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
    box-shadow: var(--shadow-panel);
  }

  .toast.error {
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    color: var(--error-text);
    box-shadow:
      var(--shadow-panel),
      inset 3px 0 0 var(--error-text);
  }

  .toast.success {
    background: var(--success-bg);
    border: 1px solid var(--success-border);
    color: var(--success-text);
    box-shadow:
      var(--shadow-panel),
      inset 3px 0 0 var(--success-text);
  }

  .toast-x {
    padding: 0 0.25rem;
    background: transparent;
    border: none;
    font-size: 1.1rem;
    line-height: 1;
    color: inherit;
    opacity: 0.7;
    box-shadow: none;
  }

  .toast-x:hover {
    opacity: 1;
    background: transparent;
  }

  .workspace {
    display: flex;
    gap: 0.6rem;
    flex: 0 0 auto;
    align-items: flex-start;
  }

  .roster-column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .roster-head {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.5rem 0.75rem;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    border-bottom: none;
    background: linear-gradient(180deg, #2a241e 0%, var(--bg-panel) 100%);
    box-shadow: inset 0 1px 0 rgba(184, 149, 74, 0.1);
  }

  .roster-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .roster-enable-all {
    padding: 0.3rem 0.65rem;
    font-size: 0.62rem;
    flex-shrink: 0;
  }

  .roster-head h2 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 0.85rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--gold-light);
  }

  .roster-hint {
    font-size: 0.68rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .list-head {
    display: grid;
    grid-template-columns: var(--roster-grid-columns);
    gap: 0 0.6rem;
    padding: 0.35rem 0.75rem;
    font-family: var(--font-display);
    font-size: 0.6rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    border-radius: 0;
    border-top: 1px solid var(--border-subtle);
    border-bottom: none;
  }

  .list-body {
    height: calc(var(--mod-row-height) * var(--mod-visible-rows));
    flex: 0 0 auto;
    overflow-y: auto;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    border-top: 1px solid var(--border-subtle);
  }

  .virtual-spacer {
    position: relative;
    width: 100%;
  }

  .virtual-window {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
  }

  .list-empty {
    margin: 0;
    padding: 2.5rem 1rem;
    text-align: center;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .list-empty p {
    margin: 0;
    font-size: 0.85rem;
    font-style: italic;
  }

  .empty-mark {
    font-size: 1.25rem;
    opacity: 0.45;
  }

  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 2.75rem 2.25rem;
    max-width: 440px;
    margin: auto;
    gap: 0.5rem;
  }

  .welcome h2 {
    margin: 0.25rem 0 0.5rem;
    font-family: var(--font-display);
    font-size: 1.35rem;
    font-weight: 700;
    color: var(--gold-light);
    letter-spacing: 0.04em;
    text-shadow: 0 1px 0 rgba(0, 0, 0, 0.35);
  }

  .welcome p {
    margin: 0 0 1rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.55;
    max-width: 34ch;
  }

  .welcome-features {
    list-style: none;
    margin: 0 0 1.35rem;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    width: 100%;
    max-width: 280px;
  }

  .welcome-features li {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.45rem 0.65rem;
    font-size: 0.78rem;
    color: var(--parchment-dim);
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    text-align: left;
  }

  .feat-icon {
    flex-shrink: 0;
    width: 1.35rem;
    text-align: center;
    color: var(--gold);
    font-size: 0.85rem;
  }

  .welcome-cta {
    padding: 0.65rem 2rem;
    min-width: 11rem;
  }
</style>
