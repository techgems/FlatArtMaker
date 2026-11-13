export type PreviewTab = 'original' | 'mapped' | 'vector' | '3d';

export const PREVIEW_TABS: ReadonlyArray<{ id: PreviewTab; label: string }> = [
  { id: 'original', label: 'Original' },
  { id: 'mapped', label: 'Mapped colours' },
  { id: 'vector', label: 'Vector (SVG)' },
  { id: '3d', label: '3D' },
];

/** Which preview is showing. */
export class PreviewTabs {
  current = $state<PreviewTab>('mapped');
}
