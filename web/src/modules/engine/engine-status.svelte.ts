import type { EngineClient } from './engine-client';

export type EnginePhase = 'loading' | 'ready' | 'failed';

/** Reactive load state of the engine, shared by any component that shows it. */
export class EngineStatus {
  phase = $state<EnginePhase>('loading');
  version = $state<string | null>(null);
  error = $state<string | null>(null);

  readonly #client: EngineClient;

  constructor(client: EngineClient) {
    this.#client = client;
  }

  async load(): Promise<void> {
    this.phase = 'loading';
    try {
      this.version = await this.#client.version();
      this.phase = 'ready';
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
      this.phase = 'failed';
    }
  }
}
