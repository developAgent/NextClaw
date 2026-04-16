import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GitBranch, Play, Plus, RefreshCw, Save, X } from "lucide-react";

interface WorkflowNode {
  id: string;
  label: string;
  node_type: "trigger" | "action" | "condition" | "delay" | "agent";
  position_x: number;
  position_y: number;
  config?: string | null;
}

interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  label?: string | null;
}

interface WorkflowVariable {
  key: string;
  value: string;
}

interface Workflow {
  id: string;
  name: string;
  description: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  variables: WorkflowVariable[];
  created_at: number;
  updated_at: number;
}

interface WorkflowNodeExecution {
  node_id: string;
  label: string;
  node_type: WorkflowNode["node_type"];
  status: "executed" | "skipped" | "failed" | string;
  message: string;
}

interface WorkflowExecutionResult {
  workflow_id: string;
  workflow_name: string;
  status: "completed" | "failed" | string;
  executed_nodes: number;
  skipped_nodes: number;
  failed_nodes: number;
  finished_at: number;
  node_results: WorkflowNodeExecution[];
}

interface WorkflowFormState {
  name: string;
  description: string;
  nodes: string;
  edges: string;
  variables: string;
}

type JsonArrayValidation<T> = {
  error: string | null;
  formatted: string;
  parsed: T[];
};

const defaultForm: WorkflowFormState = {
  name: "",
  description: "",
  nodes: "[]",
  edges: "[]",
  variables: "[]",
};

const starterNodes = JSON.stringify(
  [
    {
      id: "trigger-1",
      label: "Manual Trigger",
      node_type: "trigger",
      position_x: 120,
      position_y: 120,
      config: '{"event":"manual"}',
    },
    {
      id: "condition-1",
      label: "Check enabled",
      node_type: "condition",
      position_x: 360,
      position_y: 120,
      config: '{"expression":"enabled == true"}',
    },
    {
      id: "action-1",
      label: "Emit event",
      node_type: "action",
      position_x: 620,
      position_y: 120,
      config:
        '{"type":"emit_event","event":"workflow-demo","payload":{"source":"workflows-page"}}',
    },
  ],
  null,
  2,
);

const starterEdges = JSON.stringify(
  [
    {
      id: "edge-1",
      source: "trigger-1",
      target: "condition-1",
      label: "then",
    },
    {
      id: "edge-2",
      source: "condition-1",
      target: "action-1",
      label: "true",
    },
  ],
  null,
  2,
);

const starterVariables = JSON.stringify(
  [
    {
      key: "enabled",
      value: "true",
    },
  ],
  null,
  2,
);

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

function validateJsonArray<T>(
  raw: string,
  label: string,
): JsonArrayValidation<T> {
  const trimmed = raw.trim();
  const fallback = trimmed || "[]";

  try {
    const parsed = JSON.parse(fallback) as unknown;
    if (!Array.isArray(parsed)) {
      return {
        error: `${label} must be a JSON array.`,
        formatted: raw,
        parsed: [],
      };
    }

    return {
      error: null,
      formatted: JSON.stringify(parsed, null, 2),
      parsed: parsed as T[],
    };
  } catch (error) {
    return {
      error: `Invalid ${label.toLowerCase()} JSON: ${getErrorMessage(error)}`,
      formatted: raw,
      parsed: [],
    };
  }
}

function buildWorkflowPayload(
  form: WorkflowFormState,
  existing?: Workflow | null,
): {
  workflow: Workflow | null;
  error: string | null;
  normalizedForm: WorkflowFormState;
} {
  const nodesValidation = validateJsonArray<WorkflowNode>(form.nodes, "Nodes");
  if (nodesValidation.error) {
    return {
      workflow: null,
      error: nodesValidation.error,
      normalizedForm: { ...form, nodes: nodesValidation.formatted },
    };
  }

  const edgesValidation = validateJsonArray<WorkflowEdge>(form.edges, "Edges");
  if (edgesValidation.error) {
    return {
      workflow: null,
      error: edgesValidation.error,
      normalizedForm: {
        ...form,
        edges: edgesValidation.formatted,
        nodes: nodesValidation.formatted,
      },
    };
  }

  const variablesValidation = validateJsonArray<WorkflowVariable>(
    form.variables,
    "Variables",
  );
  if (variablesValidation.error) {
    return {
      workflow: null,
      error: variablesValidation.error,
      normalizedForm: {
        ...form,
        nodes: nodesValidation.formatted,
        edges: edgesValidation.formatted,
        variables: variablesValidation.formatted,
      },
    };
  }

  const now = Math.floor(Date.now() / 1000);
  const workflow: Workflow = {
    id: existing?.id ?? "",
    name: form.name.trim(),
    description: form.description.trim(),
    nodes: nodesValidation.parsed,
    edges: edgesValidation.parsed,
    variables: variablesValidation.parsed,
    created_at: existing?.created_at ?? now,
    updated_at: existing?.updated_at ?? now,
  };

  return {
    workflow,
    error: null,
    normalizedForm: {
      ...form,
      nodes: nodesValidation.formatted,
      edges: edgesValidation.formatted,
      variables: variablesValidation.formatted,
    },
  };
}

export default function Workflows() {
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [selectedWorkflowId, setSelectedWorkflowId] = useState<string | null>(
    null,
  );
  const [form, setForm] = useState<WorkflowFormState>(defaultForm);
  const [editingWorkflowId, setEditingWorkflowId] = useState<string | null>(
    null,
  );
  const [loading, setLoading] = useState(false);
  const [pageError, setPageError] = useState<string | null>(null);
  const [formError, setFormError] = useState<string | null>(null);
  const [executionResult, setExecutionResult] =
    useState<WorkflowExecutionResult | null>(null);
  const [runningWorkflowId, setRunningWorkflowId] = useState<string | null>(
    null,
  );

  const selectedWorkflow = useMemo(
    () =>
      workflows.find((workflow) => workflow.id === selectedWorkflowId) ?? null,
    [workflows, selectedWorkflowId],
  );

  useEffect(() => {
    void loadWorkflows();
  }, []);

  const loadWorkflows = async () => {
    try {
      setLoading(true);
      const data = await invoke<Workflow[]>("get_all_workflows");
      setWorkflows(data);
      setPageError(null);

      if (!selectedWorkflowId && data.length > 0) {
        setSelectedWorkflowId(data[0].id);
      } else if (
        selectedWorkflowId &&
        !data.some((workflow) => workflow.id === selectedWorkflowId)
      ) {
        setSelectedWorkflowId(data[0]?.id ?? null);
      }
    } catch (error) {
      console.error("Failed to load workflows:", error);
      setPageError(`Failed to load workflows: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const resetForm = () => {
    setForm(defaultForm);
    setEditingWorkflowId(null);
    setFormError(null);
  };

  const startEditing = (workflow: Workflow) => {
    setEditingWorkflowId(workflow.id);
    setSelectedWorkflowId(workflow.id);
    setForm({
      name: workflow.name,
      description: workflow.description,
      nodes: JSON.stringify(workflow.nodes, null, 2),
      edges: JSON.stringify(workflow.edges, null, 2),
      variables: JSON.stringify(workflow.variables, null, 2),
    });
    setFormError(null);
  };

  const handleLoadTemplate = () => {
    setForm({
      name: "Example workflow",
      description: "Manual trigger -> condition -> event action example.",
      nodes: starterNodes,
      edges: starterEdges,
      variables: starterVariables,
    });
    setFormError(null);
  };

  const handleCreateWorkflow = async () => {
    const payload = buildWorkflowPayload(form);
    setForm(payload.normalizedForm);

    if (payload.error || !payload.workflow) {
      setFormError(payload.error ?? "Failed to build workflow payload.");
      return;
    }

    try {
      setLoading(true);
      setFormError(null);
      const workflow = await invoke<Workflow>("create_workflow", {
        workflow: payload.workflow,
      });
      resetForm();
      setSelectedWorkflowId(workflow.id);
      await loadWorkflows();
    } catch (error) {
      console.error("Failed to create workflow:", error);
      setFormError(`Failed to create workflow: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateWorkflow = async () => {
    const existing =
      workflows.find((workflow) => workflow.id === editingWorkflowId) ?? null;
    const payload = buildWorkflowPayload(form, existing);
    setForm(payload.normalizedForm);

    if (payload.error || !payload.workflow) {
      setFormError(payload.error ?? "Failed to build workflow payload.");
      return;
    }

    try {
      setLoading(true);
      setFormError(null);
      const workflow = await invoke<Workflow>("update_workflow", {
        workflow: payload.workflow,
      });
      setSelectedWorkflowId(workflow.id);
      resetForm();
      await loadWorkflows();
    } catch (error) {
      console.error("Failed to update workflow:", error);
      setFormError(`Failed to update workflow: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRunWorkflow = async (id: string) => {
    try {
      setRunningWorkflowId(id);
      setPageError(null);
      const result = await invoke<WorkflowExecutionResult>("execute_workflow", {
        id,
      });
      setExecutionResult(result);
      setSelectedWorkflowId(result.workflow_id);
    } catch (error) {
      console.error("Failed to run workflow:", error);
      setExecutionResult(null);
      setPageError(`Failed to run workflow: ${getErrorMessage(error)}`);
    } finally {
      setRunningWorkflowId(null);
    }
  };

  const handleDeleteWorkflow = async (id: string) => {
    if (!confirm("Are you sure you want to delete this workflow?")) {
      return;
    }

    try {
      setLoading(true);
      await invoke("delete_workflow", { id });
      if (selectedWorkflowId === id) {
        setSelectedWorkflowId(null);
      }
      if (editingWorkflowId === id) {
        resetForm();
      }
      if (executionResult?.workflow_id === id) {
        setExecutionResult(null);
      }
      setPageError(null);
      await loadWorkflows();
    } catch (error) {
      console.error("Failed to delete workflow:", error);
      setPageError(`Failed to delete workflow: ${getErrorMessage(error)}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="flex flex-col gap-4 rounded-2xl border border-zinc-800 bg-zinc-900/80 p-6 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h1 className="text-2xl font-semibold">Workflows</h1>
            <p className="mt-1 text-sm text-zinc-500">
              Persist automation graphs, then manually run supported trigger,
              delay, condition, and action nodes.
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <span className="rounded-full border border-zinc-700 bg-zinc-950 px-3 py-1 text-xs uppercase tracking-[0.2em] text-zinc-300">
              {workflows.length} saved
            </span>
            <button
              onClick={() => void loadWorkflows()}
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

        <div className="grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_460px]">
          <div className="space-y-4">
            {workflows.length === 0 ? (
              <div className="flex h-80 items-center justify-center rounded-2xl border border-zinc-800 bg-zinc-900/70 text-zinc-500">
                <div className="text-center">
                  <GitBranch className="mx-auto mb-4 h-12 w-12 opacity-50" />
                  <p className="text-lg text-zinc-300">No workflows saved</p>
                  <p className="mt-2 text-sm">
                    Create your first workflow, then run it manually from the
                    list.
                  </p>
                </div>
              </div>
            ) : (
              workflows.map((workflow) => {
                const isSelected = workflow.id === selectedWorkflowId;
                const isRunning = workflow.id === runningWorkflowId;
                return (
                  <button
                    key={workflow.id}
                    type="button"
                    onClick={() => setSelectedWorkflowId(workflow.id)}
                    className={`w-full rounded-2xl border p-5 text-left transition-colors ${
                      isSelected
                        ? "border-blue-500/40 bg-blue-500/10"
                        : "border-zinc-800 bg-zinc-900/70 hover:border-zinc-700 hover:bg-zinc-900"
                    }`}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <h2 className="truncate text-lg font-semibold text-zinc-100">
                            {workflow.name}
                          </h2>
                          <span className="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
                            {workflow.nodes.length} nodes
                          </span>
                          <span className="rounded bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
                            {workflow.edges.length} edges
                          </span>
                        </div>
                        <p className="mt-2 line-clamp-2 text-sm text-zinc-400">
                          {workflow.description || "No description provided."}
                        </p>
                        <div className="mt-4 flex flex-wrap gap-4 text-xs text-zinc-500">
                          <span>
                            Updated {formatTimestamp(workflow.updated_at)}
                          </span>
                          <span>{workflow.variables.length} variables</span>
                        </div>
                      </div>
                      <div className="flex shrink-0 items-center gap-2">
                        <button
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            void handleRunWorkflow(workflow.id);
                          }}
                          disabled={isRunning}
                          className="rounded-lg border border-green-500/30 bg-green-500/10 px-3 py-2 text-xs text-green-200 transition-colors hover:bg-green-500/20 disabled:cursor-not-allowed disabled:opacity-60"
                        >
                          <span className="inline-flex items-center gap-2">
                            <Play className="h-3.5 w-3.5" />
                            {isRunning ? "Running..." : "Run"}
                          </span>
                        </button>
                        <button
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            startEditing(workflow);
                          }}
                          className="rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-xs text-zinc-100 transition-colors hover:bg-zinc-700"
                        >
                          Edit
                        </button>
                        <button
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            void handleDeleteWorkflow(workflow.id);
                          }}
                          className="rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-xs text-red-200 transition-colors hover:bg-red-500/20"
                        >
                          Delete
                        </button>
                      </div>
                    </div>
                  </button>
                );
              })
            )}
          </div>

          <div className="space-y-6">
            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h2 className="text-lg font-semibold text-zinc-100">
                    {editingWorkflowId ? "Edit workflow" : "Create workflow"}
                  </h2>
                  <p className="mt-1 text-xs text-zinc-500">
                    Edit the persisted JSON payload directly while the visual
                    builder is still pending.
                  </p>
                </div>
                <div className="flex gap-2">
                  <button
                    type="button"
                    onClick={handleLoadTemplate}
                    className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-xs text-zinc-100 transition-colors hover:bg-zinc-700"
                  >
                    <Plus className="h-4 w-4" />
                    Template
                  </button>
                  {(editingWorkflowId ||
                    form.name ||
                    form.description ||
                    form.nodes !== "[]" ||
                    form.edges !== "[]" ||
                    form.variables !== "[]") && (
                    <button
                      type="button"
                      onClick={resetForm}
                      className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-3 py-2 text-xs text-zinc-100 transition-colors hover:bg-zinc-700"
                    >
                      <X className="h-4 w-4" />
                      Reset
                    </button>
                  )}
                </div>
              </div>

              {formError && (
                <div className="mt-4 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-300">
                  {formError}
                </div>
              )}

              <div className="mt-5 space-y-4">
                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-200">
                    Name
                  </span>
                  <input
                    type="text"
                    value={form.name}
                    onChange={(event) =>
                      setForm((current) => ({
                        ...current,
                        name: event.target.value,
                      }))
                    }
                    placeholder="Nightly summary workflow"
                    className="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 text-sm text-zinc-100 outline-none transition-colors placeholder:text-zinc-500 focus:border-blue-500"
                  />
                </label>

                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-200">
                    Description
                  </span>
                  <textarea
                    value={form.description}
                    onChange={(event) =>
                      setForm((current) => ({
                        ...current,
                        description: event.target.value,
                      }))
                    }
                    rows={3}
                    placeholder="Persisted workflow graph for a manual automation path"
                    className="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 text-sm text-zinc-100 outline-none transition-colors placeholder:text-zinc-500 focus:border-blue-500"
                  />
                </label>

                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-200">
                    Nodes JSON
                  </span>
                  <textarea
                    value={form.nodes}
                    onChange={(event) =>
                      setForm((current) => ({
                        ...current,
                        nodes: event.target.value,
                      }))
                    }
                    rows={10}
                    className="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 font-mono text-xs text-zinc-100 outline-none transition-colors focus:border-blue-500"
                  />
                </label>

                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-200">
                    Edges JSON
                  </span>
                  <textarea
                    value={form.edges}
                    onChange={(event) =>
                      setForm((current) => ({
                        ...current,
                        edges: event.target.value,
                      }))
                    }
                    rows={7}
                    className="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 font-mono text-xs text-zinc-100 outline-none transition-colors focus:border-blue-500"
                  />
                </label>

                <label className="block">
                  <span className="mb-2 block text-sm font-medium text-zinc-200">
                    Variables JSON
                  </span>
                  <textarea
                    value={form.variables}
                    onChange={(event) =>
                      setForm((current) => ({
                        ...current,
                        variables: event.target.value,
                      }))
                    }
                    rows={6}
                    className="w-full rounded-xl border border-zinc-700 bg-zinc-950 px-3 py-2.5 font-mono text-xs text-zinc-100 outline-none transition-colors focus:border-blue-500"
                  />
                </label>

                <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 text-xs text-zinc-400">
                  <div className="font-medium text-zinc-200">
                    Supported manual execution
                  </div>
                  <ul className="mt-2 space-y-1">
                    <li>
                      • trigger: <code>{'{"event":"manual"}'}</code>
                    </li>
                    <li>
                      • delay: <code>{'{"duration_ms":500}'}</code>
                    </li>
                    <li>
                      • condition:{" "}
                      <code>{'{"expression":"enabled == true"}'}</code>
                    </li>
                    <li>
                      • action emit_event:{" "}
                      <code>
                        {'{"type":"emit_event","event":"demo","payload":{}}'}
                      </code>
                    </li>
                    <li>
                      • action playback_recording:{" "}
                      <code>
                        {'{"type":"playback_recording","recording_id":"..."}'}
                      </code>
                    </li>
                    <li>
                      • action execute_skill:{" "}
                      <code>
                        {
                          '{"type":"execute_skill","skill_id":"...","function":"run","args":[]}'
                        }
                      </code>
                    </li>
                  </ul>
                </div>

                <div className="flex flex-wrap gap-3">
                  <button
                    type="button"
                    onClick={() =>
                      void (editingWorkflowId
                        ? handleUpdateWorkflow()
                        : handleCreateWorkflow())
                    }
                    disabled={loading}
                    className="inline-flex items-center gap-2 rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-zinc-700"
                  >
                    {editingWorkflowId ? (
                      <Save className="h-4 w-4" />
                    ) : (
                      <Plus className="h-4 w-4" />
                    )}
                    {editingWorkflowId ? "Save workflow" : "Create workflow"}
                  </button>
                  {editingWorkflowId && (
                    <button
                      type="button"
                      onClick={resetForm}
                      className="inline-flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-800 px-4 py-2 text-sm text-zinc-100 transition-colors hover:bg-zinc-700"
                    >
                      <X className="h-4 w-4" />
                      Cancel edit
                    </button>
                  )}
                </div>
              </div>
            </section>

            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <div className="flex items-center justify-between gap-3">
                <h2 className="text-lg font-semibold text-zinc-100">
                  Selected workflow
                </h2>
                {selectedWorkflow && (
                  <button
                    type="button"
                    onClick={() => void handleRunWorkflow(selectedWorkflow.id)}
                    disabled={runningWorkflowId === selectedWorkflow.id}
                    className="inline-flex items-center gap-2 rounded-lg border border-green-500/30 bg-green-500/10 px-3 py-2 text-xs text-green-200 transition-colors hover:bg-green-500/20 disabled:cursor-not-allowed disabled:opacity-60"
                  >
                    <Play className="h-3.5 w-3.5" />
                    {runningWorkflowId === selectedWorkflow.id
                      ? "Running..."
                      : "Run workflow"}
                  </button>
                )}
              </div>
              {selectedWorkflow ? (
                <div className="mt-4 space-y-4">
                  <div>
                    <div className="text-sm font-medium text-zinc-100">
                      {selectedWorkflow.name}
                    </div>
                    <div className="mt-1 text-sm text-zinc-500">
                      {selectedWorkflow.description ||
                        "No description provided."}
                    </div>
                  </div>
                  <div className="grid gap-3 sm:grid-cols-3">
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Nodes
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {selectedWorkflow.nodes.length}
                      </div>
                    </div>
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Edges
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {selectedWorkflow.edges.length}
                      </div>
                    </div>
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Variables
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {selectedWorkflow.variables.length}
                      </div>
                    </div>
                  </div>
                  <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4 text-xs text-zinc-500">
                    <div>
                      Created {formatTimestamp(selectedWorkflow.created_at)}
                    </div>
                    <div className="mt-1">
                      Updated {formatTimestamp(selectedWorkflow.updated_at)}
                    </div>
                    <div className="mt-3 break-all">
                      Workflow ID: {selectedWorkflow.id}
                    </div>
                  </div>
                </div>
              ) : (
                <div className="mt-4 rounded-xl border border-dashed border-zinc-800 bg-zinc-950/40 px-4 py-6 text-sm text-zinc-500">
                  Select a workflow to inspect its persisted metadata and run it
                  manually.
                </div>
              )}
            </section>

            <section className="rounded-2xl border border-zinc-800 bg-zinc-900/70 p-5">
              <h2 className="text-lg font-semibold text-zinc-100">
                Last execution result
              </h2>
              {executionResult ? (
                <div className="mt-4 space-y-4">
                  <div className="flex flex-wrap items-center gap-2">
                    <span
                      className={`rounded px-2 py-0.5 text-xs ${
                        executionResult.status === "completed"
                          ? "bg-green-600/20 text-green-400"
                          : "bg-red-600/20 text-red-400"
                      }`}
                    >
                      {executionResult.status === "completed"
                        ? "Completed"
                        : "Failed"}
                    </span>
                    <span className="text-sm text-zinc-300">
                      {executionResult.workflow_name}
                    </span>
                    <span className="text-xs text-zinc-500">
                      Finished {formatTimestamp(executionResult.finished_at)}
                    </span>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-3">
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Executed
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {executionResult.executed_nodes}
                      </div>
                    </div>
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Skipped
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {executionResult.skipped_nodes}
                      </div>
                    </div>
                    <div className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4">
                      <div className="text-xs uppercase tracking-[0.2em] text-zinc-500">
                        Failed
                      </div>
                      <div className="mt-2 text-2xl font-semibold text-zinc-100">
                        {executionResult.failed_nodes}
                      </div>
                    </div>
                  </div>

                  <div className="space-y-3">
                    {executionResult.node_results.map((result) => (
                      <div
                        key={result.node_id}
                        className="rounded-xl border border-zinc-800 bg-zinc-950/70 p-4"
                      >
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-sm font-medium text-zinc-100">
                            {result.label}
                          </span>
                          <span className="rounded bg-zinc-800 px-2 py-0.5 text-[11px] uppercase text-zinc-400">
                            {result.node_type}
                          </span>
                          <span
                            className={`rounded px-2 py-0.5 text-[11px] uppercase ${
                              result.status === "executed"
                                ? "bg-green-600/20 text-green-400"
                                : result.status === "skipped"
                                  ? "bg-amber-500/20 text-amber-300"
                                  : "bg-red-600/20 text-red-400"
                            }`}
                          >
                            {result.status}
                          </span>
                        </div>
                        <p className="mt-2 text-sm text-zinc-400">
                          {result.message}
                        </p>
                      </div>
                    ))}
                  </div>
                </div>
              ) : (
                <div className="mt-4 rounded-xl border border-dashed border-zinc-800 bg-zinc-950/40 px-4 py-6 text-sm text-zinc-500">
                  Run a workflow to inspect per-node execution results here.
                </div>
              )}
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}
