import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Check,
  Keyboard,
  Plus,
  RefreshCw,
  Save,
  Trash2,
  X,
} from "lucide-react";

interface Hotkey {
  id: string;
  action: string;
  key_combination: string;
  enabled: boolean;
  created_at: number;
  updated_at: number;
}

interface RegisteredHotkey {
  id: string;
  action: string;
  key_combination: string;
}

interface RegisteredHotkeysResponse {
  registered: RegisteredHotkey[];
}

interface HotkeyFormState {
  action: string;
  key_combination: string;
}

const defaultForm: HotkeyFormState = {
  action: "",
  key_combination: "",
};

function getErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }

  return "Unknown error";
}

export default function Hotkeys() {
  const [hotkeys, setHotkeys] = useState<Hotkey[]>([]);
  const [registeredHotkeys, setRegisteredHotkeys] = useState<
    RegisteredHotkey[]
  >([]);
  const [form, setForm] = useState<HotkeyFormState>(defaultForm);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [pageError, setPageError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);

  const normalizedCombination = useMemo(
    () =>
      form.key_combination
        .split("+")
        .map((part) => part.trim())
        .filter(Boolean)
        .map((part) => part.toUpperCase())
        .join("+"),
    [form.key_combination],
  );

  useEffect(() => {
    void loadHotkeys();
    void loadRegisteredHotkeys();
  }, []);

  const loadHotkeys = async () => {
    try {
      const data = await invoke<Hotkey[]>("get_all_hotkeys");
      setHotkeys(data);
      setPageError(null);
    } catch (error) {
      console.error("Failed to load hotkeys:", error);
      setPageError(`Failed to load hotkeys: ${getErrorMessage(error)}`);
    }
  };

  const loadRegisteredHotkeys = async () => {
    try {
      const data = await invoke<RegisteredHotkeysResponse>(
        "get_registered_hotkeys",
      );
      setRegisteredHotkeys(data.registered);
      setPageError(null);
    } catch (error) {
      console.error("Failed to load registered hotkeys:", error);
      setPageError(
        `Failed to load registered hotkeys: ${getErrorMessage(error)}`,
      );
    }
  };

  const handleSyncRegistry = async () => {
    try {
      setLoading(true);
      const data = await invoke<RegisteredHotkeysResponse>("register_hotkeys");
      setRegisteredHotkeys(data.registered);
      await loadHotkeys();
      setPageError(null);
    } catch (error) {
      console.error("Failed to sync hotkeys:", error);
      setPageError(`Failed to sync hotkeys: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setForm(defaultForm);
    setEditingId(null);
    setFormError(null);
  };

  const handleCreateHotkey = async () => {
    try {
      setLoading(true);
      setFormError(null);
      await invoke<Hotkey>("add_hotkey", {
        action: form.action.trim(),
        keyCombination: form.key_combination,
      });
      await loadHotkeys();
      await loadRegisteredHotkeys();
      resetForm();
    } catch (error) {
      console.error("Failed to create hotkey:", error);
      setFormError(`Failed to create hotkey: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateHotkey = async () => {
    if (!editingId) {
      return;
    }

    try {
      setLoading(true);
      setFormError(null);
      await invoke("update_hotkey", {
        id: editingId,
        action: form.action.trim(),
        keyCombination: form.key_combination,
      });
      await loadHotkeys();
      await loadRegisteredHotkeys();
      resetForm();
    } catch (error) {
      console.error("Failed to update hotkey:", error);
      setFormError(`Failed to update hotkey: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteHotkey = async (id: string) => {
    if (!confirm("Are you sure you want to delete this hotkey?")) {
      return;
    }

    try {
      setLoading(true);
      await invoke("delete_hotkey", { id });
      await loadHotkeys();
      await loadRegisteredHotkeys();
      if (editingId === id) {
        resetForm();
      }
      setPageError(null);
    } catch (error) {
      console.error("Failed to delete hotkey:", error);
      setPageError(`Failed to delete hotkey: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleEnabled = async (hotkey: Hotkey) => {
    try {
      setLoading(true);
      await invoke("update_hotkey", {
        id: hotkey.id,
        enabled: !hotkey.enabled,
      });
      await loadHotkeys();
      await loadRegisteredHotkeys();
      setPageError(null);
    } catch (error) {
      console.error("Failed to toggle hotkey:", error);
      setPageError(`Failed to toggle hotkey: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const startEditing = (hotkey: Hotkey) => {
    setEditingId(hotkey.id);
    setForm({
      action: hotkey.action,
      key_combination: hotkey.key_combination,
    });
    setFormError(null);
  };

  const isRegistered = (id: string) =>
    registeredHotkeys.some((item) => item.id === id);

  return (
    <div className="p-6">
      <div className="mb-6 flex items-center justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold">Hotkeys</h1>
          <p className="mt-1 text-sm text-zinc-500">
            Manage shortcut definitions stored in the app database and sync
            enabled entries into the live global hotkey listener.
          </p>
        </div>
        <button
          onClick={() => void handleSyncRegistry()}
          disabled={loading}
          className="flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
        >
          <RefreshCw className="h-4 w-4" />
          Sync Runtime
        </button>
      </div>

      {pageError && (
        <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
          {pageError}
        </div>
      )}

      <div className="grid gap-6 xl:grid-cols-[minmax(0,1.4fr)_360px]">
        <div className="space-y-4">
          {hotkeys.length === 0 ? (
            <div className="flex h-64 items-center justify-center rounded-lg border border-zinc-800 bg-zinc-900 text-zinc-500">
              <div className="text-center">
                <Keyboard className="mx-auto mb-4 h-12 w-12 opacity-50" />
                <p className="text-lg">No hotkeys configured</p>
                <p className="text-sm">
                  Create your first shortcut definition to activate it in the
                  global runtime listener.
                </p>
              </div>
            </div>
          ) : (
            hotkeys.map((hotkey) => (
              <div
                key={hotkey.id}
                className="rounded-lg border border-zinc-800 bg-zinc-900 p-4"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <code className="rounded bg-zinc-800 px-2 py-1 text-sm font-medium text-zinc-100">
                        {hotkey.key_combination}
                      </code>
                      <span
                        className={`rounded px-2 py-0.5 text-xs ${hotkey.enabled ? "bg-green-600/20 text-green-400" : "bg-zinc-700 text-zinc-400"}`}
                      >
                        {hotkey.enabled ? "Enabled" : "Disabled"}
                      </span>
                      <span
                        className={`rounded px-2 py-0.5 text-xs ${isRegistered(hotkey.id) ? "bg-blue-600/20 text-blue-400" : "bg-zinc-700 text-zinc-400"}`}
                      >
                        {isRegistered(hotkey.id)
                          ? "Active in runtime"
                          : "Not active"}
                      </span>
                    </div>
                    <p className="mt-3 text-sm text-zinc-200">
                      {hotkey.action}
                    </p>
                    <p className="mt-2 text-xs text-zinc-500">
                      Updated{" "}
                      {new Date(hotkey.updated_at * 1000).toLocaleString()}
                    </p>
                  </div>
                  <div className="ml-4 flex shrink-0 items-center gap-1">
                    <button
                      onClick={() => void handleToggleEnabled(hotkey)}
                      disabled={loading}
                      className="rounded-lg p-2 transition-colors hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
                      title={hotkey.enabled ? "Disable" : "Enable"}
                    >
                      {hotkey.enabled ? (
                        <X className="h-4 w-4" />
                      ) : (
                        <Check className="h-4 w-4" />
                      )}
                    </button>
                    <button
                      onClick={() => startEditing(hotkey)}
                      disabled={loading}
                      className="rounded-lg p-2 transition-colors hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-60"
                      title="Edit"
                    >
                      <Save className="h-4 w-4" />
                    </button>
                    <button
                      onClick={() => void handleDeleteHotkey(hotkey.id)}
                      disabled={loading}
                      className="rounded-lg p-2 transition-colors hover:bg-red-600/20 hover:text-red-400 disabled:cursor-not-allowed disabled:opacity-60"
                      title="Delete"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>

        <div className="space-y-4">
          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
            <div className="mb-4 flex items-center justify-between gap-3">
              <div>
                <h2 className="text-lg font-semibold">
                  {editingId ? "Edit Hotkey" : "New Hotkey"}
                </h2>
                <p className="mt-1 text-xs text-zinc-500">
                  Use combinations like CTRL+SHIFT+K or ALT+SPACE.
                </p>
              </div>
              {!editingId && (
                <div className="rounded-full bg-blue-600/15 p-2 text-blue-400">
                  <Plus className="h-4 w-4" />
                </div>
              )}
            </div>

            {formError && (
              <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                {formError}
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label className="mb-1 block text-sm text-zinc-400">
                  Action
                </label>
                <input
                  type="text"
                  value={form.action}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      action: event.target.value,
                    }))
                  }
                  placeholder="Open launcher"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div>
                <label className="mb-1 block text-sm text-zinc-400">
                  Key Combination
                </label>
                <input
                  type="text"
                  value={form.key_combination}
                  onChange={(event) =>
                    setForm((current) => ({
                      ...current,
                      key_combination: event.target.value,
                    }))
                  }
                  placeholder="CTRL+SHIFT+K"
                  className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 font-mono focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                {normalizedCombination && (
                  <p className="mt-2 text-xs text-zinc-500">
                    Normalized preview: {normalizedCombination}
                  </p>
                )}
              </div>

              <div className="flex justify-end gap-2">
                <button
                  onClick={resetForm}
                  className="rounded-lg bg-zinc-800 px-4 py-2 transition-colors hover:bg-zinc-700"
                >
                  {editingId ? "Cancel" : "Reset"}
                </button>
                <button
                  onClick={() =>
                    void (editingId
                      ? handleUpdateHotkey()
                      : handleCreateHotkey())
                  }
                  disabled={
                    loading ||
                    !form.action.trim() ||
                    !form.key_combination.trim()
                  }
                  className="rounded-lg bg-blue-600 px-4 py-2 transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                >
                  {loading
                    ? "Saving..."
                    : editingId
                      ? "Update Hotkey"
                      : "Create Hotkey"}
                </button>
              </div>
            </div>
          </div>

          <div className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
            <h2 className="text-lg font-semibold">Active Runtime Hotkeys</h2>
            <p className="mt-1 text-xs text-zinc-500">
              This reflects enabled hotkeys currently loaded into the global
              listener. Supported actions: toggle_main_window, show_main_window,
              hide_main_window.
            </p>
            <div className="mt-4 space-y-3">
              {registeredHotkeys.length === 0 ? (
                <p className="text-sm text-zinc-500">
                  No hotkeys are active in runtime.
                </p>
              ) : (
                registeredHotkeys.map((hotkey) => (
                  <div
                    key={hotkey.id}
                    className="rounded-lg bg-zinc-950 px-3 py-2"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <code className="text-xs text-blue-300">
                        {hotkey.key_combination}
                      </code>
                      <span className="rounded bg-blue-600/20 px-2 py-0.5 text-[11px] text-blue-400">
                        Active
                      </span>
                    </div>
                    <p className="mt-1 text-sm text-zinc-300">
                      {hotkey.action}
                    </p>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
