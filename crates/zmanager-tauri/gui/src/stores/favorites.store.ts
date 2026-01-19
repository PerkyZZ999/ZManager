/**
 * Favorites Store - Zustand store for Quick Access favorites
 *
 * Manages favorites list with CRUD operations and reordering.
 */

import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

// ============================================================================
// Types
// ============================================================================

/** Favorite item DTO from backend */
export interface FavoriteDto {
  id: string;
  name: string;
  path: string;
  order: number;
  icon: string | null;
  is_valid: boolean;
}

/** IPC response wrapper */
interface IpcResponse<T> {
  ok: boolean;
  data?: T;
  error?: string;
}

// ============================================================================
// Store State
// ============================================================================

interface FavoritesState {
  /** List of favorites */
  favorites: FavoriteDto[];
  /** Loading state */
  isLoading: boolean;
  /** Error message */
  error: string | null;

  // Actions
  loadFavorites: () => Promise<void>;
  addFavorite: (name: string, path: string, icon?: string) => Promise<boolean>;
  removeFavorite: (id: string) => Promise<boolean>;
  removeFavoriteByPath: (path: string) => Promise<boolean>;
  reorderFavorites: (ids: string[]) => Promise<boolean>;
  /** Check if a path is already a favorite */
  isFavorite: (path: string) => boolean;
  /** Toggle favorite status for a path */
  toggleFavorite: (name: string, path: string, icon?: string) => Promise<boolean>;
}

// ============================================================================
// Store Implementation
// ============================================================================

export const useFavoritesStore = create<FavoritesState>((set, get) => ({
  favorites: [],
  isLoading: false,
  error: null,

  loadFavorites: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await invoke<IpcResponse<FavoriteDto[]>>("zmanager_get_favorites");
      if (response.ok && response.data) {
        set({ favorites: response.data, isLoading: false });
      } else {
        set({ error: response.error ?? "Failed to load favorites", isLoading: false });
      }
    } catch (err) {
      set({ error: String(err), isLoading: false });
    }
  },

  addFavorite: async (name: string, path: string, icon?: string) => {
    try {
      const response = await invoke<IpcResponse<FavoriteDto>>("zmanager_add_favorite", {
        name,
        path,
        icon: icon ?? null,
      });
      if (response.ok && response.data) {
        // Append to list
        set((state) => ({
          favorites: [...state.favorites, response.data as FavoriteDto],
        }));
        return true;
      }
      set({ error: response.error ?? "Failed to add favorite" });
      return false;
    } catch (err) {
      set({ error: String(err) });
      return false;
    }
  },

  removeFavorite: async (id: string) => {
    try {
      const response = await invoke<IpcResponse<boolean>>("zmanager_remove_favorite", { id });
      if (response.ok) {
        // Remove from list
        set((state) => ({
          favorites: state.favorites.filter((f) => f.id !== id),
        }));
        return true;
      }
      set({ error: response.error ?? "Failed to remove favorite" });
      return false;
    } catch (err) {
      set({ error: String(err) });
      return false;
    }
  },

  reorderFavorites: async (ids: string[]) => {
    // Optimistically update order
    const currentFavorites = get().favorites;
    const reorderedFavorites = ids
      .map((id) => currentFavorites.find((f) => f.id === id))
      .filter((f): f is FavoriteDto => f !== undefined);

    set({ favorites: reorderedFavorites });

    try {
      const response = await invoke<IpcResponse<null>>("zmanager_reorder_favorites", { ids });
      if (!response.ok) {
        // Revert on failure
        set({
          favorites: currentFavorites,
          error: response.error ?? "Failed to reorder favorites",
        });
        return false;
      }
      return true;
    } catch (err) {
      // Revert on error
      set({ favorites: currentFavorites, error: String(err) });
      return false;
    }
  },

  isFavorite: (path: string) => {
    const normalized = path.toLowerCase().replace(/\\/g, "/").replace(/\/$/, "");
    return get().favorites.some(
      (f) => f.path.toLowerCase().replace(/\\/g, "/").replace(/\/$/, "") === normalized
    );
  },

  removeFavoriteByPath: async (path: string) => {
    const normalized = path.toLowerCase().replace(/\\/g, "/").replace(/\/$/, "");
    const favorite = get().favorites.find(
      (f) => f.path.toLowerCase().replace(/\\/g, "/").replace(/\/$/, "") === normalized
    );
    if (!favorite) return false;
    return get().removeFavorite(favorite.id);
  },

  toggleFavorite: async (name: string, path: string, icon?: string) => {
    if (get().isFavorite(path)) {
      return get().removeFavoriteByPath(path);
    }
    return get().addFavorite(name, path, icon);
  },
}));
