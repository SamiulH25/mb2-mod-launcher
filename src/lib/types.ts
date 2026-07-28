export interface SubModuleInfo {
  id: string;
  name: string;
  version: string | null;
  singleplayer: boolean;
  multiplayer: boolean;
  depended_modules: DependedModule[];
  depended_module_metadatas: DependedModuleMetadata[];
  dll_names: string[];
  url: string | null;
  folder_name: string;
}

export interface DependedModule {
  id: string;
  version: string | null;
  optional: boolean;
}

export interface DependedModuleMetadata {
  id: string;
  order: string | null;
  version: string | null;
  optional: boolean;
}

export interface InstalledModule {
  info: SubModuleInfo;
  path: string;
  source: "game" | "workshop" | "manual";
  workshop_item_id?: string | null;
}

export interface ModuleState {
  module: InstalledModule;
  enabled: boolean;
  position: number;
}

export interface GamePathsSnapshot {
  game_root: string;
  modules_dir: string;
  launcher_data: string;
  workshop_dir: string | null;
  workshop_dirs: string[];
  proton_prefix: string | null;
}

export interface AppState {
  paths: GamePathsSnapshot;
  modules: ModuleState[];
  warnings: string[];
}

export interface UnblockResult {
  files_processed: number;
  files_unblocked: number;
  errors: string[];
}
