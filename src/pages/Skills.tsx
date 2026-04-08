import { useState, useEffect, useRef } from 'react';
import { Plus, Database, Search, Trash2, ToggleRight, FileCode, Download, Upload } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface Skill {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  permissions: any[];
  enabled: boolean;
}

interface SkillManifest {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  permissions: any[];
  api_version: string;
  entry_point: string;
  config_schema?: any;
}

export default function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [showMarketplace, setShowMarketplace] = useState(false);
  const [loading, setLoading] = useState(false);
  const [showInstallModal, setShowInstallModal] = useState(false);
  const [manifestInput, setManifestInput] = useState('');
  const [selectedManifest, setSelectedManifest] = useState<SkillManifest | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    try {
      const data = await invoke<SkillManifest[]>('wasm_list_skills');
      const skillList = data.map(s => ({
        ...s,
        enabled: true, // All loaded skills are considered enabled
      }));
      setSkills(skillList as Skill[]);
    } catch (error) {
      console.error('Failed to load skills:', error);
    }
  };

  const handleUninstallSkill = async (id: string) => {
    if (!confirm('Are you sure you want to uninstall this skill?')) return;

    try {
      await invoke('wasm_unregister_skill', { skillId: id });
      await loadSkills();
    } catch (error) {
      console.error('Failed to uninstall skill:', error);
      alert('Failed to uninstall skill');
    }
  };

  const handleInstallFromManifest = async () => {
    try {
      const manifest = JSON.parse(manifestInput);
      setSelectedManifest(manifest);
      setShowInstallModal(false);
      // Would typically prompt for WASM file here
      alert('Manifest parsed. Please select the WASM file.');
    } catch (error) {
      alert('Invalid manifest JSON');
    }
  };

  const handleSelectWasmFile = async (event: React.ChangeEvent<HTMLInputElement>) => {
    try {
      const file = event.target.files?.[0];
      if (!file) return;

      // Read the WASM file and convert to base64
      const wasmContent = await file.arrayBuffer();
      const wasmBase64 = btoa(String.fromCharCode(...new Uint8Array(wasmContent)));

      // Install the skill
      setLoading(true);
      await invoke('wasm_register_skill', {
        wasmBase64,
        manifestJson: JSON.stringify(selectedManifest),
        permissionsJson: JSON.stringify({ granted: [], denied: [] }),
      });

      await loadSkills();
      setSelectedManifest(null);
      setLoading(false);
      setShowInstallModal(false);
    } catch (error) {
      console.error('Failed to install skill:', error);
      alert('Failed to install skill');
      setLoading(false);
    }
  };

  const filteredSkills = skills.filter(skill =>
    skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    skill.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Skills</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowMarketplace(!showMarketplace)}
            className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-colors ${
              showMarketplace ? 'bg-zinc-700 hover:bg-zinc-600' : 'bg-blue-600 hover:bg-blue-700'
            }`}
          >
            <Database className="w-4 h-4" />
            {showMarketplace ? 'Installed' : 'Marketplace'}
          </button>
          {!showMarketplace && (
            <button
              onClick={() => setShowInstallModal(true)}
              className="flex items-center gap-2 px-4 py-2 bg-zinc-700 hover:bg-zinc-600 rounded-lg transition-colors"
            >
              <Plus className="w-4 h-4" />
              Install
            </button>
          )}
        </div>
      </div>

      {showMarketplace ? (
        <>
          <div className="mb-6">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-zinc-500" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search skills..."
                className="w-full bg-zinc-800 border border-zinc-700 rounded-lg pl-10 pr-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          <div className="text-center text-zinc-500 py-12">
            <Database className="w-16 h-16 mx-auto mb-4 opacity-50" />
            <p className="text-lg">Skill Marketplace</p>
            <p className="text-sm">Browse and install community skills</p>
            <p className="text-xs mt-2 text-zinc-600">Marketplace coming soon</p>
          </div>
        </>
      ) : (
        <>
          {skills.length === 0 ? (
            <div className="flex items-center justify-center h-64 text-zinc-500">
              <div className="text-center">
                <FileCode className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p className="text-lg">No skills installed</p>
                <p className="text-sm">Install skills from WASM files or the marketplace</p>
              </div>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredSkills.map((skill) => (
                <div key={skill.id} className="bg-zinc-900 border border-zinc-800 rounded-lg p-4">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <h3 className="font-medium">{skill.name}</h3>
                      <p className="text-xs text-zinc-500">v{skill.version} by {skill.author}</p>
                    </div>
                    <div className={`px-2 py-1 rounded text-xs ${
                      skill.enabled ? 'bg-green-600/20 text-green-400' : 'bg-zinc-700 text-zinc-400'
                    }`}>
                      {skill.enabled ? 'Enabled' : 'Disabled'}
                    </div>
                  </div>
                  <p className="text-sm text-zinc-400 mb-3">{skill.description}</p>

                  {skill.permissions.length > 0 && (
                    <div className="mb-3">
                      <p className="text-xs text-zinc-500 mb-1">Permissions:</p>
                      <div className="flex flex-wrap gap-1">
                        {skill.permissions.slice(0, 3).map((perm, idx) => (
                          <span key={idx} className="px-2 py-0.5 bg-zinc-800 rounded text-xs text-zinc-400">
                            {perm.permission_type}
                          </span>
                        ))}
                        {skill.permissions.length > 3 && (
                          <span className="px-2 py-0.5 bg-zinc-800 rounded text-xs text-zinc-500">
                            +{skill.permissions.length - 3} more
                          </span>
                        )}
                      </div>
                    </div>
                  )}

                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => {}}
                      className="flex-1 flex items-center justify-center gap-1 px-3 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors text-sm"
                    >
                      <ToggleRight className="w-4 h-4" />
                      {skill.enabled ? 'Disable' : 'Enable'}
                    </button>
                    <button
                      onClick={() => handleUninstallSkill(skill.id)}
                      className="p-2 hover:bg-red-600/20 hover:text-red-400 rounded-lg transition-colors"
                      title="Uninstall"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      )}

      {/* Install Modal */}
      {showInstallModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-zinc-900 rounded-lg p-6 max-w-lg w-full">
            <h2 className="text-xl font-semibold mb-4">Install Skill</h2>

            {!selectedManifest ? (
              <>
                <div className="mb-4">
                  <label className="block text-sm text-zinc-400 mb-1">
                    Paste skill manifest (JSON)
                  </label>
                  <textarea
                    value={manifestInput}
                    onChange={(e) => setManifestInput(e.target.value)}
                    placeholder='{
  "id": "com.example.my-skill",
  "name": "My Skill",
  "version": "1.0.0",
  "description": "A sample skill",
  "author": "Your Name",
  "permissions": []
}'
                    rows={12}
                    className="w-full bg-zinc-800 border border-zinc-700 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none font-mono text-sm"
                  />
                </div>
                <div className="flex justify-end gap-2">
                  <button
                    onClick={() => setShowInstallModal(false)}
                    className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleInstallFromManifest}
                    disabled={!manifestInput.trim()}
                    className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-zinc-700 disabled:cursor-not-allowed rounded-lg transition-colors"
                  >
                    Next
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="bg-zinc-800 rounded-lg p-4 mb-4">
                  <h3 className="font-medium mb-2">{selectedManifest.name}</h3>
                  <p className="text-sm text-zinc-400 mb-2">{selectedManifest.description}</p>
                  <p className="text-xs text-zinc-500">
                    Version: {selectedManifest.version} | Author: {selectedManifest.author}
                  </p>
                </div>

                <div className="mb-4">
                  <label className="block text-sm text-zinc-400 mb-2">
                    Select WASM file
                  </label>
                  <input
                    ref={fileInputRef}
                    type="file"
                    accept=".wasm"
                    onChange={handleSelectWasmFile}
                    disabled={loading}
                    className="hidden"
                  />
                  <button
                    onClick={() => fileInputRef.current?.click()}
                    disabled={loading}
                    className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-zinc-800 hover:bg-zinc-700 border-2 border-dashed border-zinc-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <Download className="w-4 h-4" />
                    {loading ? 'Installing...' : 'Select WASM File'}
                  </button>
                </div>

                <div className="flex justify-end gap-2">
                  <button
                    onClick={() => {
                      setSelectedManifest(null);
                      setManifestInput('');
                    }}
                    className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                  >
                    Back
                  </button>
                  <button
                    onClick={() => setShowInstallModal(false)}
                    className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  );
}