/**
 * The UI's only way into the engine.
 * Implementations: `createWorkerEngineClient` (WASM in a worker); fakes in tests.
 */
export interface EngineClient {
  version(): Promise<string>;
  dispose(): void;
}
