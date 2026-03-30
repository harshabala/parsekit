<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getSetting, setSetting } from "./lib/store";
  import { runParse, type ParseEvent } from "./lib/sidecar";
  import "./index.css";

  // State
  let inputDir = $state("");
  let outputDir = $state("");
  let format = $state("md");
  let isParsing = $state(false);
  let progress = $state(0);
  let ocrEnabled = $state(true);
  let ocrLanguage = $state("eng");
  let files = $state<{ name: string; status: string; path?: string }[]>([]);
  let recentBatches = $state<{ name: string; date: string; path: string }[]>([]);

  onMount(async () => {
    inputDir = await getSetting("inputDir", "");
    outputDir = await getSetting("outputDir", "");
    format = await getSetting("format", "md");
    ocrEnabled = await getSetting("ocrEnabled", true);
    ocrLanguage = await getSetting("ocrLanguage", "eng");
    recentBatches = await getSetting("recentBatches", []);
  });

  async function pickInput() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      inputDir = selected as string;
      await setSetting("inputDir", inputDir);
    }
  }

  async function pickOutput() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      outputDir = selected as string;
      await setSetting("outputDir", outputDir);
    }
  }

  async function startParse() {
    if (!inputDir || !outputDir) return;
    
    isParsing = true;
    progress = 0;
    files = [];
    
    try {
      await runParse({ 
        inputDir, 
        outputDir, 
        format: format as any, 
        ocrLanguage: ocrEnabled ? ocrLanguage : undefined 
      }, (event) => {
        if (event.type === "progress") {
          const existingIndex = files.findIndex(f => f.name === event.file);
          if (existingIndex !== -1) {
            files[existingIndex].status = event.status || "";
            if (event.path) files[existingIndex].path = event.path;
          } else {
            files = [{ name: event.file!, status: event.status!, path: event.path }, ...files];
          }
          
          const completedCount = files.filter(f => f.status === "completed").length;
          progress = files.length > 0 ? (completedCount / files.length) * 100 : 0;
        } else if (event.type === "done") {
          isParsing = false;
          progress = 100;
          addToHistory();
        } else if (event.type === "error") {
          isParsing = false;
          console.error(event.message);
        }
      });
    } catch (e) {
      isParsing = false;
      console.error(e);
    }
  }

  async function addToHistory() {
    const newBatch = {
      name: inputDir.split("/").pop() || "Batch",
      date: new Date().toLocaleString(),
      path: outputDir
    };
    recentBatches = [newBatch, ...recentBatches.filter(b => b.path !== outputDir).slice(0, 4)];
    await setSetting("recentBatches", recentBatches);
  }

  async function copyToClipboard() {
    const lastFile = files.find(f => f.status === "completed");
    if (lastFile && lastFile.path) {
      await writeText(lastFile.path);
    }
  }

  async function openFolder(path: string) {
    const { Command } = await import("@tauri-apps/plugin-shell");
    await Command.create("open", [path]).spawn();
  }
</script>

<div id="app">
  <header>
    <span>ParseDock</span>
  </header>

  <main>
    <div class="section">
      <div class="section-title">Configuration</div>
      <div class="card">
        <div class="row">
          <button class="secondary" onclick={pickInput}>Input Folder</button>
          <span class="path-preview">{inputDir || "Not selected..."}</span>
        </div>
        <div class="row">
          <button class="secondary" onclick={pickOutput}>Output Folder</button>
          <span class="path-preview">{outputDir || "Not selected..."}</span>
        </div>
        
        <div class="row">
          <span>Format</span>
          <select bind:value={format}>
            <option value="md">Markdown (.md)</option>
            <option value="json">JSON (.json)</option>
            <option value="txt">Plain Text (.txt)</option>
          </select>
        </div>

        <div class="row">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input type="checkbox" bind:checked={ocrEnabled} id="ocr-toggle" />
            <label for="ocr-toggle">OCR (Tesseract)</label>
          </div>
          {#if ocrEnabled}
            <select bind:value={ocrLanguage} style="width: 80px;">
              <option value="eng">English</option>
              <option value="spa">Spanish</option>
              <option value="fra">French</option>
              <option value="deu">German</option>
            </select>
          {/if}
        </div>
      </div>
    </div>

    {#if isParsing || (progress > 0 && progress < 100) || (progress === 100 && files.length > 0)}
      <div class="section">
        <div class="section-title">Processing</div>
        <div class="card progress-container">
          <div class="row">
            <span>{isParsing ? "Refining Documents..." : "Parsing Complete"}</span>
            <span>{Math.round(progress)}%</span>
          </div>
          <div class="progress-bar">
            <div class="progress-fill" style="width: {progress}%"></div>
          </div>
          <div class="file-list">
            {#each files as file}
              <div class="file-item">
                <span title={file.name} style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 200px;">
                  {file.name}
                </span>
                <span class="status-{file.status}">{file.status}</span>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <div class="section" style="margin-top: auto;">
      <button 
        disabled={isParsing || !inputDir || !outputDir} 
        onclick={startParse}
        style="width: 100%; height: 36px; font-size: 14px;"
      >
        {isParsing ? "Parsing..." : "Run Parse"}
      </button>
      {#if progress === 100 && !isParsing && files.length > 0}
        <div class="row" style="margin-top: 8px;">
          <button class="secondary" style="flex: 1" onclick={() => openFolder(outputDir)}>Open Output</button>
          <button class="secondary" style="flex: 1" onclick={copyToClipboard}>Copy Last Path</button>
        </div>
      {/if}
    </div>

    {#if recentBatches.length > 0 && !isParsing}
      <div class="section">
        <div class="section-title">Recent Batches</div>
        <div class="card" style="padding: 0; overflow: hidden;">
          {#each recentBatches as batch}
            <button 
              type="button"
              class="history-item" 
              onclick={() => openFolder(batch.path)} 
              style="width: 100%; text-align: left; background: transparent; border: none; padding: 8px; border-bottom: 1px solid var(--border-color); cursor: pointer; display: flex; flex-direction: column; gap: 2px;"
            >
              <span class="history-name" style="color: var(--text-color); font-weight: 500;">{batch.name}</span>
              <span class="history-meta" style="color: #8e8e93; font-size: 10px;">{batch.date}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </main>
</div>
