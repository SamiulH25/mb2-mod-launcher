<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { AppState, ModuleState } from "./lib/types";
  import CoatOfArms from "./lib/components/CoatOfArms.svelte";
  import CalradiaBanner from "./lib/components/CalradiaBanner.svelte";
  import ModCard from "./lib/components/ModCard.svelte";
  import LoadOrderTimeline from "./lib/components/LoadOrderTimeline.svelte";
  import QuickActionsPanel from "./lib/components/QuickActionsPanel.svelte";

  let appState: AppState | null = $state(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let statusMessage = $state<string | null>(null);
  let searchQuery = $state("");
  let dragIndex: number | null = $state(null);
  let dragOverIndex: number | null = $state(null);
  let expandedId: string | null = $state(null);

  const filteredModules = $derived(
    appState
      ? appState.modules.filter((m) => {
          if (!searchQuery.trim()) return true;
          const q = searchQuery.toLowerCase();
          return (
            m.module.info.name.toLowerCase().includes(q) ||
            m.module.info.id.toLowerCase().includes(q)
          );
        })
      : [],
  );

  const enabledCount = $derived(
    appState?.modules.filter((m) => m.enabled).length ?? 0,
  );

  const totalCount = $derived(appState?.modules.length ?? 0);

  $effect(() => {
    if (!statusMessage) return;
    const timer = setTimeout(() => {
      statusMessage = null;
    }, 3500);
    return () => clearTimeout(timer);
  });

  async function detectGame() {
    loading = true;
    error = null;
    statusMessage = null;
    try {
      appState = await invoke<AppState>("detect_game");
      statusMessage = `Found ${appState.modules.length} modules`;
    } catch (e) {
      error = String(e);
      appState = null;
    } finally {
      loading = false;
    }
  }

  async function refreshModules() {
    if (!appState) return;
    loading = true;
    error = null;
    try {
      appState = await invoke<AppState>("refresh_modules");
      statusMessage = "Modules refreshed";
    } catch (e) {
      error = String(e);
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
      error = String(e);
    }
  }

  async function autoSort() {
    if (!appState) return;
    loading = true;
    error = null;
    try {
      appState = await invoke<AppState>("auto_sort_modules");
      statusMessage = "Load order sorted";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function saveLoadOrder() {
    if (!appState) return;
    loading = true;
    error = null;
    try {
      await invoke("save_load_order");
      statusMessage = "Saved to LauncherData.xml";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function unblockDlls() {
    if (!appState) return;
    loading = true;
    error = null;
    try {
      const results = await invoke<Array<{ files_unblocked: number }>>("unblock_dlls");
      const total = results.reduce((sum, r) => sum + r.files_unblocked, 0);
      statusMessage = `Unblocked ${total} DLL file(s)`;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function onDragStart(index: number) {
    dragIndex = index;
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (dragOverIndex !== index) {
      dragOverIndex = index;
    }
  }

  function onDragLeave() {
    dragOverIndex = null;
  }

  async function onDrop(targetIndex: number) {
    dragOverIndex = null;
    if (!appState || dragIndex === null || dragIndex === targetIndex) {
      dragIndex = null;
      return;
    }

    const ids = appState.modules.map((m) => m.module.info.id);
    const [moved] = ids.splice(dragIndex, 1);
    ids.splice(targetIndex, 0, moved);
    dragIndex = null;

    try {
      appState = await invoke<AppState>("reorder_modules", { moduleIds: ids });
    } catch (e) {
      error = String(e);
    }
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function dismissStatus() {
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
            <h2>Module Roster</h2>
            <span class="roster-hint">Click a row for details · drag to reorder</span>
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

          <div class="list-body panel" role="list" aria-label="Installed modules">
            {#each filteredModules as module, index (module.module.info.id)}
              <ModCard
                {module}
                {index}
                expanded={expandedId === module.module.info.id}
                dragging={dragIndex === index}
                dragOver={dragOverIndex === index}
                ontoggle={() => toggleModule(module)}
                onexpand={() => toggleExpand(module.module.info.id)}
                ondragstart={() => onDragStart(index)}
                ondragover={(e) => onDragOver(e, index)}
                ondragleave={onDragLeave}
                ondrop={() => onDrop(index)}
              />
            {:else}
              <p class="list-empty">No modules match your search.</p>
            {/each}
          </div>
        </section>

        <QuickActionsPanel
          bind:searchQuery
          gamePath={appState.paths.game_root}
          configPath={appState.paths.launcher_data}
          {loading}
          warnings={appState.warnings}
          onAutoSort={autoSort}
          onUnblockDlls={unblockDlls}
          onSave={saveLoadOrder}
          onRefresh={refreshModules}
        />
      </div>

      <LoadOrderTimeline modules={appState.modules} />
    {:else}
      <section class="welcome panel-inset">
        <CoatOfArms size={72} />
        <h2>Rally Your Mods</h2>
        <p>
          Detect your Steam install to muster modules from <code>SubModule.xml</code>,
          arrange your campaign load order, and deploy to <code>LauncherData.xml</code>.
        </p>
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
    padding: 0.5rem 0.7rem;
    border-radius: var(--radius-sm);
    font-size: 0.8125rem;
  }

  .toast.error {
    background: var(--error-bg);
    border: 1px solid var(--error-border);
    color: var(--error-text);
  }

  .toast.success {
    background: var(--success-bg);
    border: 1px solid var(--success-border);
    color: var(--success-text);
  }

  .toast-x {
    padding: 0 0.25rem;
    background: transparent;
    border: none;
    font-size: 1.1rem;
    line-height: 1;
    color: inherit;
    opacity: 0.7;
  }

  .toast-x:hover {
    opacity: 1;
    background: transparent;
  }

  .workspace {
    display: flex;
    gap: 0.6rem;
    flex: 1;
    min-height: 0;
  }

  .roster-column {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .roster-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding: 0.45rem 0.75rem;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
    border-bottom: none;
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
    grid-template-columns: 2.25rem 1fr 3.5rem auto auto 1.25rem 2.5rem;
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
    flex: 1;
    overflow-y: auto;
    min-height: 0;
    border-radius: 0 0 var(--radius-md) var(--radius-md);
    border-top: 1px solid var(--border-subtle);
  }

  .list-empty {
    margin: 0;
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
    font-style: italic;
  }

  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 2.5rem 2rem;
    max-width: 420px;
    margin: auto;
    gap: 0.35rem;
  }

  .welcome h2 {
    margin: 1rem 0 0.5rem;
    font-family: var(--font-display);
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--gold-light);
    letter-spacing: 0.04em;
  }

  .welcome p {
    margin: 0 0 1.25rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.55;
  }

  .welcome-cta {
    padding: 0.6rem 1.75rem;
  }
</style>
