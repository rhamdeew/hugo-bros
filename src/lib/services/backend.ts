// Backend service for Tauri command invocations
import { invoke } from '@tauri-apps/api/core';
import type {
  Post,
  Page,
  Draft,
  ImageInfo,
  HexoConfig,
  FrontmatterConfig,
  AppConfig,
  CommandOutput
} from '$lib/types';

export class BackendService {
  private projectPath: string | null = null;

  constructor() {
    this.loadProjectPath();
  }

  private loadProjectPath() {
    this.projectPath = localStorage.getItem('projectPath') || null;
  }

  setProjectPath(path: string) {
    this.projectPath = path;
    localStorage.setItem('projectPath', path);
  }

  getProjectPath(): string | null {
    return this.projectPath;
  }

  private ensureProject(): string {
    if (!this.projectPath) {
      throw new Error('No project selected. Please select a project folder first.');
    }
    return this.projectPath;
  }

  // ====================
  // Project Commands
  // ====================

  async selectProjectFolder(): Promise<string> {
    const path = await invoke<string>('select_project_folder');
    this.setProjectPath(path);
    return path;
  }

  async getProjectConfig(): Promise<HexoConfig> {
    const projectPath = this.ensureProject();
    return invoke<HexoConfig>('get_project_config', { projectPath });
  }

  async getFrontmatterConfig(): Promise<FrontmatterConfig> {
    const projectPath = this.ensureProject();
    return invoke<FrontmatterConfig>('get_frontmatter_config', { projectPath });
  }

  // ====================
  // Posts Commands
  // ====================

  async listPosts(): Promise<Post[]> {
    const projectPath = this.ensureProject();
    return invoke<Post[]>('list_posts', { projectPath });
  }

  async getPost(postId: string): Promise<Post> {
    const projectPath = this.ensureProject();
    return invoke<Post>('get_post', { projectPath, postId });
  }

  async savePost(post: Post): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('save_post', { projectPath, post });
  }

  async getPage(pageId: string): Promise<Page> {
    const projectPath = this.ensureProject();
    return invoke<Page>('get_page', { projectPath, pageId });
  }

  async savePage(page: Page): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('save_page', { projectPath, page });
  }

  async deletePage(pageId: string): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('delete_page', { projectPath, pageId });
  }

  async createPost(title: string): Promise<Post> {
    const projectPath = this.ensureProject();
    return invoke<Post>('create_post', { projectPath, title });
  }

  async deletePost(postId: string): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('delete_post', { projectPath, postId });
  }

  // ====================
  // Pages Commands
  // ====================

  async createPage(title: string): Promise<Page> {
    const projectPath = this.ensureProject();
    return invoke<Page>('create_page', { projectPath, title });
  }

  async listPages(): Promise<Page[]> {
    const projectPath = this.ensureProject();
    return invoke<Page[]>('list_pages', { projectPath });
  }

  // ====================
  // Drafts Commands
  // ====================

  async createDraft(title: string): Promise<Draft> {
    const projectPath = this.ensureProject();
    return invoke<Draft>('create_draft', { projectPath, title });
  }

  async getDraft(draftId: string): Promise<Draft> {
    const projectPath = this.ensureProject();
    return invoke<Draft>('get_draft', { projectPath, draftId });
  }

  async saveDraft(draft: Draft): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('save_draft', { projectPath, draft });
  }

  async deleteDraft(draftId: string): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('delete_draft', { projectPath, draftId });
  }

  async listDrafts(): Promise<Draft[]> {
    const projectPath = this.ensureProject();
    return invoke<Draft[]>('list_drafts', { projectPath });
  }

  // ====================
  // Images Commands
  // ====================

  async listImages(): Promise<ImageInfo[]> {
    const projectPath = this.ensureProject();
    return invoke<ImageInfo[]>('list_images', { projectPath });
  }

  async copyImageToProject(sourcePath: string): Promise<string> {
    const projectPath = this.ensureProject();
    return invoke<string>('copy_image_to_project', { projectPath, sourcePath });
  }

  async deleteImage(imagePath: string): Promise<void> {
    const projectPath = this.ensureProject();
    await invoke('delete_image', { projectPath, imagePath });
  }

  // ====================
  // App Config Commands
  // ====================

  async getAppConfig(): Promise<AppConfig> {
    return invoke<AppConfig>('get_app_config');
  }

  async saveAppConfig(config: AppConfig): Promise<void> {
    await invoke('save_app_config', { config });
  }

  // ====================
  // Hexo Server Commands
  // ====================

  async runHexoCommand(command: string): Promise<CommandOutput> {
    const projectPath = this.ensureProject();
    return invoke<CommandOutput>('run_hexo_command', { projectPath, command });
  }

  async startHexoServer(): Promise<string> {
    const projectPath = this.ensureProject();
    return invoke<string>('start_hexo_server', { projectPath });
  }

  async stopHexoServer(serverId: string): Promise<void> {
    await invoke('stop_hexo_server', { serverId });
  }

  async isHexoServerRunning(): Promise<boolean> {
    const projectPath = this.ensureProject();
    return invoke<boolean>('is_hexo_server_running', { projectPath });
  }
}

// Singleton instance
export const backend = new BackendService();
