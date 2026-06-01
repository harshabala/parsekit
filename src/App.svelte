<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { getSetting, setSetting } from "./lib/store";
  import { runParse, type ParseEvent } from "./lib/sidecar";
  import type { OutputFormat, FileProgress, BatchResult } from "./lib/types";
import { MAX_RECENT_BATCHES } from "./lib/types";
  import FolderPicker from "./components/FolderPicker.svelte";
  import FormatSelector from "./components/FormatSelector.svelte";
  import ProgressList from "./components/ProgressList.svelte";
  import RecentBatches from "./components/RecentBatches.svelte";
  import AboutScreen from "./components/AboutScreen.svelte";
  import "./index.css";

  // State
  let inputDir = $state("");
  let outputDir = $state("");
  let format = $state<OutputFormat>("md");
  let isParsing = $state(false);
  let ocrEnabled = $state(true);
  let ocrLanguage = $state("eng");
  let workers = $state(4);
  let files = $state<FileProgress[]>([]);
  let totalFiles = $state(0);
  let recentBatches = $state<BatchResult[]>([]);
  let showAbout = $state(false);
  let inputFileCount = $state<number | null>(null);
  let errorMsg = $state<string | null>(null);

  let showProgress = $derived(isParsing || files.length > 0);

  onMount(async () => {
    inputDir = await getSetting("inputDir", "");
    outputDir = await getSetting("outputDir", "");
    format = await getSetting<OutputFormat>("format", "md");
    ocrEnabled = await getSetting("ocrEnabled", true);
    ocrLanguage = await getSetting("ocrLanguage", "eng");
    const savedWorkers = await getSetting<number>("workers", 0);
    recentBatches = await getSetting<BatchResult[]>("recentBatches", []);

    try {
      const sysInfo = await invoke<{ optimalWorkers: number; isAppleSilicon: boolean }>("get_system_info");
      workers = savedWorkers > 0 ? savedWorkers : sysInfo.optimalWorkers;
    } catch {
      workers = savedWorkers > 0 ? savedWorkers : 4;
    }

    if (inputDir) {
      scanInput(inputDir);
    }
  });

  async function scanInput(path: string) {
    try {
      const scanned = await invoke<string[]>("scan_directory", { path });
      inputFileCount = scanned.length;
    } catch {
      inputFileCount = null;
    }
  }

  async function handleInputSelect(path: string) {
    inputDir = path;
    await setSetting("inputDir", inputDir);
    scanInput(path);
  }

  async function handleOutputSelect(path: string) {
    outputDir = path;
    await setSetting("outputDir", outputDir);
  }

  async function handleFormatChange(f: OutputFormat) {
    format = f;
    await setSetting("format", format);
  }

  async function handleWorkersChange(e: Event) {
    const target = e.target as HTMLInputElement;
    workers = parseInt(target.value, 10);
    await setSetting("workers", workers);
  }

  async function startParse() {
    if (!inputDir || !outputDir) return;

    try {
      await invoke("trigger_haptic");
    } catch {}

    isParsing = true;
    files = [];
    totalFiles = 0;
    errorMsg = null;

    try {
      await runParse({
        inputDir,
        outputDir,
        format,
        ocrEnabled,
        ocrLanguage,
        workers,
      }, (event: ParseEvent) => {
        if (event.type === "start") {
          totalFiles = event.total || 0;
        } else if (event.type === "progress") {
          const name = event.file || "";
          let status: FileProgress["status"] = "pending";
          if (event.status === "completed" || event.status === "done") status = "done";
          else if (event.status === "parsing") status = "parsing";
          else if (event.status === "error") status = "error";
          else if (event.status === "skipped") status = "skipped";

          const existingIndex = files.findIndex(f => f.name === name);
          if (existingIndex !== -1) {
            files[existingIndex] = { ...files[existingIndex], status, path: event.path || files[existingIndex].path, error: event.error };
          } else {
            files = [{ name, status, path: event.path, error: event.error }, ...files];
          }
        } else if (event.type === "done") {
          isParsing = false;
          totalFiles = totalFiles || files.length;
          addToHistory();
        } else if (event.type === "error") {
          isParsing = false;
          errorMsg = event.message || "Parsing failed.";
          console.error(event.message);
        }
      });
    } catch (e) {
      isParsing = false;
      errorMsg = e instanceof Error ? e.message : String(e);
      console.error(e);
    }
  }

  async function addToHistory() {
    const parsed = files.filter(f => f.status === "done").length;
    const errors = files.filter(f => f.status === "error").length;
    const newBatch: BatchResult = {
      id: crypto.randomUUID(),
      timestamp: new Date().toISOString(),
      inputDir,
      outputDir,
      format,
      fileCount: files.length,
      parsed,
      errors,
    };
    recentBatches = [newBatch, ...recentBatches.slice(0, MAX_RECENT_BATCHES - 1)];
    await setSetting("recentBatches", recentBatches);
  }

  async function copyToClipboard() {
    const lastFile = files.find(f => f.status === "done");
    if (lastFile && lastFile.path) {
      try {
        const bytes = await invoke<number[]>("copy_file_to_clipboard", { path: lastFile.path });
        const content = new TextDecoder().decode(new Uint8Array(bytes));
        await writeText(content);
        try {
          await invoke("trigger_haptic");
        } catch {}
      } catch {
        await writeText(lastFile.path);
      }
    }
  }

  async function openFolder(path: string) {
    try {
      await invoke("open_in_finder", { path });
      try {
        await invoke("trigger_haptic");
      } catch {}
    } catch {
      // Fallback to shell open
      const { Command } = await import("@tauri-apps/plugin-shell");
      await Command.create("open", [path]).spawn();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "r") {
      e.preventDefault();
      if (!isParsing && inputDir && outputDir) {
        startParse();
      }
    }
    if (e.key === "Escape" && showAbout) {
      showAbout = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div id="app">
  <header>
    <span>ParseDock</span>
    <button class="icon-btn" onclick={() => (showAbout = !showAbout)} title="About ParseDock">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="8" cy="8" r="7" stroke="currentColor" stroke-width="1.5"/>
        <path d="M7 6.5C7 5.67 7.67 5 8.5 5C9.33 5 10 5.67 10 6.5C10 7.17 9.5 7.5 9 7.8C8.7 7.97 8.5 8.17 8.5 8.5V9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
        <circle cx="8.5" cy="11" r="0.75" fill="currentColor"/>
      </svg>
    </button>
  </header>

  <main>
    <div class="section">
      <div class="section-title">Configuration</div>
      <div class="card">
        <FolderPicker label="Input Folder" value={inputDir} onSelect={handleInputSelect} />
        {#if inputFileCount !== null}
          {#if inputFileCount === 0}
            <div class="file-count-preview">No supported documents in this folder.</div>
          {:else}
            <div class="file-count-preview">{inputFileCount} file{inputFileCount !== 1 ? "s" : ""} found</div>
          {/if}
        {/if}
        <FolderPicker label="Output Folder" value={outputDir} onSelect={handleOutputSelect} />

        <div class="row">
          <span>Format</span>
          <FormatSelector value={format} onChange={handleFormatChange} />
        </div>
        {#if format !== "json"}
          <div class="file-count-preview caption-hint">Spreadsheets always export as JSON.</div>
        {/if}

        <div class="row">
          <div style="display: flex; align-items: center; gap: 8px;">
            <input type="checkbox" bind:checked={ocrEnabled} id="ocr-toggle" />
            <label for="ocr-toggle">OCR (Tesseract)</label>
          </div>
          {#if ocrEnabled}
            <select bind:value={ocrLanguage} style="width: 100px;">
              <option value="eng">English</option>
              <option value="spa">Spanish</option>
              <option value="fra">French</option>
              <option value="deu">German</option>
              <option value="por">Portuguese</option>
              <option value="ita">Italian</option>
              <option value="chi_sim">Chinese (Simplified)</option>
              <option value="chi_tra">Chinese (Traditional)</option>
              <option value="jpn">Japanese</option>
              <option value="kor">Korean</option>
              <option value="rus">Russian</option>
              <option value="ara">Arabic</option>
              <option value="nld">Dutch</option>
              <option value="pol">Polish</option>
              <option value="vie">Vietnamese</option>
              <option value="tha">Thai</option>
              <option value="tur">Turkish</option>
              <option value="hin">Hindi</option>
            </select>
          {/if}
        </div>

        <div class="row">
          <label for="workers-slider">Workers</label>
          <div class="workers-control">
            <input
              type="range"
              id="workers-slider"
              min="1"
              max="8"
              bind:value={workers}
              oninput={handleWorkersChange}
              class="workers-slider"
            />
            <span class="workers-value">{workers}</span>
          </div>
        </div>
      </div>
    </div>

    {#if showProgress}
      <ProgressList {files} total={totalFiles || files.length} {isParsing} />
    {/if}

    <div class="section" style="margin-top: auto;">
      <button
        disabled={isParsing || !inputDir || !outputDir || inputFileCount === 0}
        onclick={startParse}
        style="width: 100%; height: 36px; font-size: 14px;"
      >
        {isParsing ? "Parsing..." : "Run Parse"}
      </button>
      {#if errorMsg}
        <div class="error-banner" role="alert">{errorMsg}</div>
      {/if}
      {#if !isParsing && files.length > 0 && files.some(f => f.status === "done")}
        <div class="row" style="margin-top: 8px;">
          <button class="secondary" style="flex: 1" onclick={() => openFolder(outputDir)}>Open Output</button>
          <button class="secondary" style="flex: 1" onclick={copyToClipboard}>Copy Last File</button>
        </div>
      {/if}
    </div>

    {#if !isParsing}
      <RecentBatches batches={recentBatches} onOpenFolder={openFolder} />
    {/if}
  </main>

  {#if showAbout}
    <AboutScreen onClose={() => (showAbout = false)} />
  {/if}
</div>
