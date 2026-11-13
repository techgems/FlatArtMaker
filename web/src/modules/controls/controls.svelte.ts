import type { Advanced, ArtBounds, Settings } from '../engine';
import { fit } from './utils/clamp';
import { LIMITS } from './utils/limits';
import { longestSide, printSize, scaleFor, type PrintSize, type SizeSide } from './utils/size';

/** Where the controls read settings from and send changes to (the session). */
export interface SettingsSource {
  readonly settings: Settings | null;
  readonly bounds: ArtBounds | null;
  update(settings: Settings): void;
}

/** The Advanced controls that take a number. */
export type AdvancedNumber = Exclude<keyof Advanced, 'fillBaseHoles'>;

/**
 * The left panel's values, and the rules between them: limits, the locked
 * aspect ratio, B = 0 meaning no base, and the thin-feature threshold kept
 * while the fix is off. Every change becomes a whole new `Settings`.
 */
export class Controls {
  /** The threshold to bring back when "Fix thin features" is turned on again. */
  #threshold = $state<number>(LIMITS.minFeature.min);
  readonly #source: SettingsSource;

  constructor(source: SettingsSource) {
    this.#source = source;
  }

  get ready(): boolean {
    return this.#source.settings !== null && this.#source.bounds !== null;
  }

  get colourHeight(): number {
    return this.#settings().colourHeight;
  }

  /** 0 when there is no base. */
  get baseHeight(): number {
    return this.#settings().baseHeight ?? 0;
  }

  /** B = 0: the base-only controls are disabled. */
  get noBase(): boolean {
    return this.baseHeight === 0;
  }

  get size(): PrintSize {
    return printSize(this.#settings().scale, this.#bounds());
  }

  get longestSide(): number {
    return longestSide(this.#settings().scale, this.#bounds());
  }

  get fixThin(): boolean {
    return this.#settings().minFeature !== undefined;
  }

  get threshold(): number {
    return this.#settings().minFeature ?? this.#threshold;
  }

  advanced<K extends keyof Advanced>(key: K): Advanced[K] {
    return this.#settings().advanced[key];
  }

  setColourHeight(value: number): void {
    const colourHeight = fit(value, LIMITS.colourHeight);
    if (colourHeight !== null) this.#update({ colourHeight });
  }

  setBaseHeight(value: number): void {
    const base = fit(value, LIMITS.baseHeight);
    if (base !== null) this.#update({ baseHeight: base === 0 ? undefined : base });
  }

  setSize(side: SizeSide, mm: number): void {
    const scale = scaleFor(side, mm, this.#bounds());
    if (scale !== null) this.#update({ scale });
  }

  setFixThin(on: boolean): void {
    const current = this.#settings().minFeature;
    if (current !== undefined) this.#threshold = current;
    this.#update({ minFeature: on ? this.#threshold : undefined });
  }

  setThreshold(value: number): void {
    const threshold = fit(value, LIMITS.minFeature);
    if (threshold === null) return;
    this.#threshold = threshold;
    if (this.fixThin) this.#update({ minFeature: threshold });
  }

  setAdvanced(key: AdvancedNumber, value: number): void {
    const fitted = fit(value, LIMITS[key]);
    if (fitted !== null) this.#updateAdvanced({ [key]: fitted });
  }

  setFillBaseHoles(on: boolean): void {
    this.#updateAdvanced({ fillBaseHoles: on });
  }

  #settings(): Settings {
    const settings = this.#source.settings;
    if (settings === null) throw new Error('No image is open');
    return settings;
  }

  #bounds(): ArtBounds {
    const bounds = this.#source.bounds;
    if (bounds === null) throw new Error('No image is open');
    return bounds;
  }

  #update(change: Partial<Settings>): void {
    this.#source.update({ ...this.#settings(), ...change });
  }

  #updateAdvanced(change: Partial<Advanced>): void {
    const settings = this.#settings();
    this.#update({ advanced: { ...settings.advanced, ...change } });
  }
}
