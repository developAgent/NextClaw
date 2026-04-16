import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  CirclePause,
  Clapperboard,
  History,
  Play,
  Plus,
  RefreshCw,
  Square,
  Trash2,
} from "lucide-react";

interface RecordingEvent {
  id: string;
  event_type: string;
  payload: string;
  created_at: number;
}

interface Recording {
  id: string;
  name: string;
  description: string;
  status: string;
  events: RecordingEvent[];
  created_at: number;
  updated_at: number;
}

interface RecordedEventDraft {
  event_type: string;
  payload: string;
}

type RecordingState =
  | { state: "idle" }
  | { state: "stopped" }
  | { state: "recording"; started_at: number }
  | { state: "paused"; paused_at: number }
  | { state: "playing_back"; recording_id: string; started_at: number };

interface PlaybackResult {
  recording_id: string;
  replayed_events: number;
  skipped_events: number;
  failed_events: number;
  warnings: string[];
  finished_at: number;
}

interface RecordingFormState {
  name: string;
  description: string;
}

const defaultRecordingForm: RecordingFormState = {
  name: "",
  description: "",
};

const defaultEventForm: RecordedEventDraft = {
  event_type: "",
  payload: "",
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

function formatTimestamp(value: number): string {
  return new Date(value * 1000).toLocaleString();
}

function describeState(state: RecordingState): string {
  switch (state.state) {
    case "recording":
      return "Recording";
    case "paused":
      return "Paused";
    case "playing_back":
      return "Playing back";
    case "stopped":
      return "Stopped";
    case "idle":
    default:
      return "Idle";
  }
}

export default function Recorder() {
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const [currentRecording, setCurrentRecording] = useState<Recording | null>(
    null,
  );
  const [recordingState, setRecordingState] = useState<RecordingState>({
    state: "idle",
  });
  const [recordingForm, setRecordingForm] =
    useState<RecordingFormState>(defaultRecordingForm);
  const [eventForm, setEventForm] =
    useState<RecordedEventDraft>(defaultEventForm);
  const [selectedRecordingId, setSelectedRecordingId] = useState<string | null>(
    null,
  );
  const [pageError, setPageError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [eventError, setEventError] = useState<string | null>(null);
  const [playbackResult, setPlaybackResult] = useState<PlaybackResult | null>(
    null,
  );
  const [loading, setLoading] = useState(false);

  const selectedRecording = useMemo(
    () =>
      recordings.find((recording) => recording.id === selectedRecordingId) ??
      null,
    [recordings, selectedRecordingId],
  );

  useEffect(() => {
    void refreshAll();
  }, []);

  const refreshAll = async () => {
    try {
      setLoading(true);
      const [state, current, saved] = await Promise.all([
        invoke<RecordingState>("get_recording_state"),
        invoke<Recording | null>("get_current_recording"),
        invoke<Recording[]>("list_recordings"),
      ]);

      setRecordingState(state);
      setCurrentRecording(current);
      setRecordings(saved);
      setPageError(null);

      if (!selectedRecordingId && saved.length > 0) {
        setSelectedRecordingId(saved[0].id);
      } else if (
        selectedRecordingId &&
        !saved.some((recording) => recording.id === selectedRecordingId)
      ) {
        setSelectedRecordingId(saved[0]?.id ?? null);
      }
    } catch (error) {
      console.error("Failed to load recorder state:", error);
      setPageError(`Failed to load recorder state: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const resetRecordingForm = () => {
    setRecordingForm(defaultRecordingForm);
    setFormError(null);
  };

  const handleStartRecording = async () => {
    try {
      setLoading(true);
      setFormError(null);
      setPlaybackResult(null);
      const recording = await invoke<Recording>("start_recording", {
        name: recordingForm.name.trim(),
        description: recordingForm.description.trim(),
      });
      setCurrentRecording(recording);
      resetRecordingForm();
      await refreshAll();
    } catch (error) {
      console.error("Failed to start recording:", error);
      setFormError(`Failed to start recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleStopRecording = async () => {
    try {
      setLoading(true);
      setPlaybackResult(null);
      const recording = await invoke<Recording>("stop_recording");
      setSelectedRecordingId(recording.id);
      setCurrentRecording(null);
      await refreshAll();
    } catch (error) {
      console.error("Failed to stop recording:", error);
      setPageError(`Failed to stop recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handlePauseRecording = async () => {
    try {
      setLoading(true);
      await invoke("pause_recording");
      await refreshAll();
    } catch (error) {
      console.error("Failed to pause recording:", error);
      setPageError(`Failed to pause recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleResumeRecording = async () => {
    try {
      setLoading(true);
      await invoke("resume_recording");
      await refreshAll();
    } catch (error) {
      console.error("Failed to resume recording:", error);
      setPageError(`Failed to resume recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAddEvent = async () => {
    try {
      setLoading(true);
      setEventError(null);
      await invoke("add_event", {
        event: {
          event_type: eventForm.event_type.trim(),
          payload: eventForm.payload,
        },
      });
      setEventForm(defaultEventForm);
      await refreshAll();
    } catch (error) {
      console.error("Failed to add event:", error);
      setEventError(`Failed to add event: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handlePlayback = async (recordingId: string) => {
    try {
      setLoading(true);
      const result = await invoke<PlaybackResult>("playback_recording", {
        id: recordingId,
      });
      setPlaybackResult(result);
      await refreshAll();
    } catch (error) {
      console.error("Failed to playback recording:", error);
      setPageError(`Failed to playback recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteRecording = async (recordingId: string) => {
    if (!confirm("Are you sure you want to delete this recording?")) {
      return;
    }

    try {
      setLoading(true);
      await invoke("delete_recording", { id: recordingId });
      if (selectedRecordingId === recordingId) {
        setSelectedRecordingId(null);
      }
      setPlaybackResult(null);
      await refreshAll();
    } catch (error) {
      console.error("Failed to delete recording:", error);
      setPageError(`Failed to delete recording: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const canStart =
    recordingState.state === "idle" || recordingState.state === "stopped";
  const canPause = recordingState.state === "recording";
  const canResume = recordingState.state === "paused";
  const canStop =
    recordingState.state === "recording" || recordingState.state === "paused";
  const canAddEvent = recordingState.state === "recording";

  return (
    <div className="p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="flex flex-col gap-4 rounded-2xl border border-zinc-800 bg-zinc-900/80 p-6 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h1 className="text-2xl font-semibold">Recorder</h1>
            <p className="mt-1 text-sm text-zinc-500">
              Capture automation events, persist completed recordings, and
              replay saved flows.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <span className="rounded-full border border-zinc-700 bg-zinc-950 px-3 py-1 text-xs uppercase tracking-[0.2em] text-zinc-300">
              {describeState(recordingState)}
            </span>
            <button
              onClick={() => void refreshAll()}
              disabled={loading}
              className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
            >
              <RefreshCw
                className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
              />
              Refresh
            </button>
          </div>
        </div>

        {pageError && (
          <div className="rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
            {pageError}
          </div>
        )}

        {playbackResult && (
          <div className="rounded-xl border border-blue-500/30 bg-blue-500/10 px-4 py-3 text-sm text-blue-200">
            <div>
              Playback finished for {playbackResult.recording_id} at{" "}
              {formatTimestamp(playbackResult.finished_at)}.
            </div>
            <div className="mt-1 text-blue-100/90">
              Executed {playbackResult.replayed_events}, skipped{" "}
              {playbackResult.skipped_events}, failed{" "}
              {playbackResult.failed_events}.
            </div>
            {playbackResult.warnings.length > 0 && (
              <ul className="mt-2 list-disc space-y-1 pl-5 text-blue-100/80">
                {playbackResult.warnings.map((warning, index) => (
                  <li key={`${warning}-${index}`}>{warning}</li>
                ))}
              </ul>
            )}
          </div>
        )}

        <div className="grid gap-6 xl:grid-cols-[minmax(0,1.25fr)_420px]">
          <div className="space-y-6">
            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-lg font-semibold">Active session</h2>
                  <p className="mt-1 text-xs text-zinc-500">
                    Start a recording, append captured events, then persist it
                    by stopping the session.
                  </p>
                </div>
                <div className="rounded-full bg-blue-600/15 p-2 text-blue-400">
                  <Clapperboard className="h-4 w-4" />
                </div>
              </div>

              <div className="mt-5 grid gap-4 lg:grid-cols-2">
                <div className="space-y-4">
                  {formError && (
                    <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                      {formError}
                    </div>
                  )}

                  <div>
                    <label className="mb-1 block text-sm text-zinc-400">
                      Recording name
                    </label>
                    <input
                      value={recordingForm.name}
                      onChange={(event) =>
                        setRecordingForm((current) => ({
                          ...current,
                          name: event.target.value,
                        }))
                      }
                      placeholder="Checkout smoke test"
                      className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm outline-none transition-colors focus:border-blue-500"
                    />
                  </div>

                  <div>
                    <label className="mb-1 block text-sm text-zinc-400">
                      Description
                    </label>
                    <textarea
                      value={recordingForm.description}
                      onChange={(event) =>
                        setRecordingForm((current) => ({
                          ...current,
                          description: event.target.value,
                        }))
                      }
                      rows={4}
                      placeholder="Capture the click path and text input for a common workflow."
                      className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm outline-none transition-colors focus:border-blue-500"
                    />
                  </div>

                  <div className="flex flex-wrap gap-2">
                    <button
                      onClick={() => void handleStartRecording()}
                      disabled={
                        loading || !canStart || !recordingForm.name.trim()
                      }
                      className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                    >
                      <Play className="h-4 w-4" />
                      Start
                    </button>
                    <button
                      onClick={() => void handlePauseRecording()}
                      disabled={loading || !canPause}
                      className="inline-flex items-center gap-2 rounded-lg bg-zinc-800 px-4 py-2 text-sm transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      <CirclePause className="h-4 w-4" />
                      Pause
                    </button>
                    <button
                      onClick={() => void handleResumeRecording()}
                      disabled={loading || !canResume}
                      className="inline-flex items-center gap-2 rounded-lg bg-zinc-800 px-4 py-2 text-sm transition-colors hover:bg-zinc-700 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                      <Play className="h-4 w-4" />
                      Resume
                    </button>
                    <button
                      onClick={() => void handleStopRecording()}
                      disabled={loading || !canStop}
                      className="inline-flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                    >
                      <Square className="h-4 w-4" />
                      Stop
                    </button>
                  </div>
                </div>

                <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4">
                  <div className="text-sm font-medium text-zinc-200">
                    Current recording
                  </div>
                  {currentRecording ? (
                    <div className="mt-3 space-y-3 text-sm text-zinc-300">
                      <div>
                        <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                          Name
                        </div>
                        <div className="mt-1 text-zinc-100">
                          {currentRecording.name}
                        </div>
                      </div>
                      <div>
                        <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                          Description
                        </div>
                        <div className="mt-1 whitespace-pre-wrap text-zinc-300">
                          {currentRecording.description || "No description"}
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-3 text-xs text-zinc-400">
                        <div>
                          <div className="uppercase tracking-[0.2em] text-zinc-500">
                            Events
                          </div>
                          <div className="mt-1 text-sm text-zinc-100">
                            {currentRecording.events.length}
                          </div>
                        </div>
                        <div>
                          <div className="uppercase tracking-[0.2em] text-zinc-500">
                            Updated
                          </div>
                          <div className="mt-1 text-sm text-zinc-100">
                            {formatTimestamp(currentRecording.updated_at)}
                          </div>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="mt-6 text-sm text-zinc-500">
                      No recording session is active.
                    </div>
                  )}
                </div>
              </div>
            </section>

            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-lg font-semibold">Capture event</h2>
                  <p className="mt-1 text-xs text-zinc-500">
                    Add normalized event payloads to the current in-memory
                    recording session.
                  </p>
                </div>
                <div className="rounded-full bg-blue-600/15 p-2 text-blue-400">
                  <Plus className="h-4 w-4" />
                </div>
              </div>

              {eventError && (
                <div className="mt-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                  {eventError}
                </div>
              )}

              <div className="mt-5 grid gap-4 lg:grid-cols-[220px_minmax(0,1fr)_auto]">
                <div>
                  <label className="mb-1 block text-sm text-zinc-400">
                    Event type
                  </label>
                  <input
                    value={eventForm.event_type}
                    onChange={(event) =>
                      setEventForm((current) => ({
                        ...current,
                        event_type: event.target.value,
                      }))
                    }
                    placeholder="click"
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm outline-none transition-colors focus:border-blue-500"
                  />
                </div>

                <div>
                  <label className="mb-1 block text-sm text-zinc-400">
                    Payload
                  </label>
                  <textarea
                    value={eventForm.payload}
                    onChange={(event) =>
                      setEventForm((current) => ({
                        ...current,
                        payload: event.target.value,
                      }))
                    }
                    rows={3}
                    placeholder='{"selector":"#submit","x":420,"y":218}'
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-sm font-mono outline-none transition-colors focus:border-blue-500"
                  />
                </div>

                <div className="flex items-end">
                  <button
                    onClick={() => void handleAddEvent()}
                    disabled={
                      loading ||
                      !canAddEvent ||
                      !eventForm.event_type.trim() ||
                      !eventForm.payload.trim()
                    }
                    className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                  >
                    <Plus className="h-4 w-4" />
                    Add event
                  </button>
                </div>
              </div>

              <div className="mt-4 space-y-2 text-xs text-zinc-500">
                <div>
                  Events are only accepted while the recorder is actively in the
                  recording state.
                </div>
                <div>
                  Supported playback event types: <code>text</code>,{" "}
                  <code>key</code>, <code>mouse_move</code>,{" "}
                  <code>mouse_button</code>, and <code>delay</code>.
                </div>
                <div>
                  Example payloads: <code>{'{"text":"hello"}'}</code>,{" "}
                  <code>{'{"key":"enter"}'}</code>,{" "}
                  <code>{'{"x":420,"y":218}'}</code>,{" "}
                  <code>{'{"button":"left","x":420,"y":218}'}</code>, and{" "}
                  <code>{'{"duration_ms":500}'}</code>.
                </div>
              </div>
            </section>
          </div>

          <aside className="space-y-6">
            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-lg font-semibold">Saved recordings</h2>
                  <p className="mt-1 text-xs text-zinc-500">
                    Persisted recordings are loaded from SQLite and can be
                    replayed or deleted.
                  </p>
                </div>
                <div className="rounded-full bg-blue-600/15 p-2 text-blue-400">
                  <History className="h-4 w-4" />
                </div>
              </div>

              <div className="mt-4 space-y-3">
                {recordings.length === 0 ? (
                  <div className="rounded-xl border border-zinc-800 bg-zinc-950/60 px-4 py-8 text-center text-sm text-zinc-500">
                    No recordings saved yet.
                  </div>
                ) : (
                  recordings.map((recording) => {
                    const isSelected = recording.id === selectedRecordingId;
                    return (
                      <button
                        key={recording.id}
                        onClick={() => setSelectedRecordingId(recording.id)}
                        className={`w-full rounded-xl border p-4 text-left transition-colors ${
                          isSelected
                            ? "border-blue-500/40 bg-blue-600/10"
                            : "border-zinc-800 bg-zinc-950/60 hover:bg-zinc-800/70"
                        }`}
                      >
                        <div className="flex items-start justify-between gap-3">
                          <div className="min-w-0 flex-1">
                            <div className="truncate text-sm font-medium text-zinc-100">
                              {recording.name}
                            </div>
                            <div className="mt-1 line-clamp-2 text-xs text-zinc-500">
                              {recording.description || "No description"}
                            </div>
                          </div>
                          <span className="rounded bg-zinc-800 px-2 py-0.5 text-[11px] text-zinc-300">
                            {recording.events.length} events
                          </span>
                        </div>
                        <div className="mt-3 text-xs text-zinc-500">
                          Updated {formatTimestamp(recording.updated_at)}
                        </div>
                      </button>
                    );
                  })
                )}
              </div>
            </section>

            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="text-lg font-semibold text-zinc-100">
                Recording detail
              </div>
              {selectedRecording ? (
                <div className="mt-4 space-y-4">
                  <div>
                    <div className="text-sm font-medium text-zinc-100">
                      {selectedRecording.name}
                    </div>
                    <p className="mt-1 whitespace-pre-wrap text-sm text-zinc-400">
                      {selectedRecording.description || "No description"}
                    </p>
                  </div>

                  <div className="grid grid-cols-2 gap-3 text-xs text-zinc-500">
                    <div className="rounded-lg bg-zinc-950/60 p-3">
                      <div className="uppercase tracking-[0.2em]">Status</div>
                      <div className="mt-1 text-sm text-zinc-100">
                        {selectedRecording.status}
                      </div>
                    </div>
                    <div className="rounded-lg bg-zinc-950/60 p-3">
                      <div className="uppercase tracking-[0.2em]">Events</div>
                      <div className="mt-1 text-sm text-zinc-100">
                        {selectedRecording.events.length}
                      </div>
                    </div>
                  </div>

                  <div className="flex flex-wrap gap-2">
                    <button
                      onClick={() => void handlePlayback(selectedRecording.id)}
                      disabled={loading}
                      className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                    >
                      <Play className="h-4 w-4" />
                      Replay
                    </button>
                    <button
                      onClick={() =>
                        void handleDeleteRecording(selectedRecording.id)
                      }
                      disabled={loading}
                      className="inline-flex items-center gap-2 rounded-lg bg-red-600/90 px-4 py-2 text-sm transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                    >
                      <Trash2 className="h-4 w-4" />
                      Delete
                    </button>
                  </div>

                  <div className="space-y-3">
                    {selectedRecording.events.length === 0 ? (
                      <div className="rounded-lg bg-zinc-950/60 px-4 py-6 text-sm text-zinc-500">
                        This recording has no captured events.
                      </div>
                    ) : (
                      selectedRecording.events.map((event) => (
                        <article
                          key={event.id}
                          className="rounded-xl border border-zinc-800 bg-zinc-950/60 p-4"
                        >
                          <div className="flex items-center justify-between gap-3">
                            <span className="rounded bg-zinc-800 px-2 py-1 text-xs uppercase tracking-[0.2em] text-zinc-200">
                              {event.event_type}
                            </span>
                            <span className="text-xs text-zinc-500">
                              {formatTimestamp(event.created_at)}
                            </span>
                          </div>
                          <pre className="mt-3 overflow-x-auto whitespace-pre-wrap break-words rounded-lg bg-zinc-900 p-3 text-xs text-zinc-300">
                            {event.payload}
                          </pre>
                        </article>
                      ))
                    )}
                  </div>
                </div>
              ) : (
                <div className="mt-4 rounded-xl border border-zinc-800 bg-zinc-950/60 px-4 py-8 text-center text-sm text-zinc-500">
                  Select a saved recording to inspect its event history.
                </div>
              )}
            </section>
          </aside>
        </div>
      </div>
    </div>
  );
}
