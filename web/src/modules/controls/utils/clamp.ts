import type { Limit } from './limits';

/** Decimal places in `step`, so snapped values print without float noise. */
function decimals(step: number): number {
  const [, fraction = ''] = String(step).split('.');
  return fraction.length;
}

/**
 * `value` brought into `limit`: clamped to its range and snapped to its
 * step. `null` for something that isn't a number, so the caller keeps the
 * old value.
 */
export function fit(value: number, limit: Limit): number | null {
  if (!Number.isFinite(value)) return null;
  const clamped = Math.min(limit.max, Math.max(limit.min, value));
  const snapped = limit.min + Math.round((clamped - limit.min) / limit.step) * limit.step;
  return Math.min(limit.max, Number(snapped.toFixed(decimals(limit.step))));
}
