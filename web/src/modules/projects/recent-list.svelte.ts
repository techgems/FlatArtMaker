import type { ProjectSummary } from './project-store';

/** The Recent list's row state: which one is being renamed, or awaiting delete confirmation. */
export class RecentList {
  renaming = $state<string | null>(null);
  draft = $state('');
  confirming = $state<string | null>(null);

  startRename(project: ProjectSummary): void {
    this.confirming = null;
    this.renaming = project.id;
    this.draft = project.name;
  }

  askDelete(id: string): void {
    this.renaming = null;
    this.confirming = id;
  }

  cancel(): void {
    this.renaming = null;
    this.confirming = null;
  }
}
