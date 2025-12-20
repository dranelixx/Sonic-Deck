import { invoke } from "@tauri-apps/api/core";

interface WaveformData {
  peaks: number[];
  duration_ms: number;
}

interface QueueItem {
  filePath: string;
  numPeaks: number;
  resolve: (data: WaveformData) => void;
  reject: (error: any) => void;
}

class WaveformQueue {
  private queue: QueueItem[] = [];
  private isProcessing = false;
  private readonly delay = 20; // 20ms between requests

  async add(filePath: string, numPeaks: number): Promise<WaveformData> {
    return new Promise((resolve, reject) => {
      this.queue.push({ filePath, numPeaks, resolve, reject });
      this.process();
    });
  }

  private async process() {
    if (this.isProcessing || this.queue.length === 0) return;

    this.isProcessing = true;

    while (this.queue.length > 0) {
      const item = this.queue.shift();
      if (!item) break;

      try {
        const data = await invoke<WaveformData>("get_waveform", {
          filePath: item.filePath,
          numPeaks: item.numPeaks,
        });
        item.resolve(data);
      } catch (error) {
        item.reject(error);
      }

      // Small delay between requests
      if (this.queue.length > 0) {
        await new Promise((resolve) => setTimeout(resolve, this.delay));
      }
    }

    this.isProcessing = false;
  }
}

// Global singleton queue
export const waveformQueue = new WaveformQueue();
