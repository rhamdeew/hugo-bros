// Core types for Hexo Blog Editor

export interface Post {
  id: string;
  title: string;
  date: string;
  content: string;
  frontmatter: Frontmatter;
  filePath: string;
  createdAt: number;
  modifiedAt: number;
}

export interface Frontmatter {
  title: string;
  date: string;
  tags: string[];
  categories: string[];
  updated?: string;
  comments?: boolean;
  layout?: string;
  permalink?: string;
  description?: string;
  customFields?: Record<string, unknown>;
}

export interface FrontmatterFieldConfig {
  name: string;
  label?: string;
  type: string;
  description?: string;
  ui?: {
    placeholder?: string;
    rows?: number;
  };
}

export interface FrontmatterFieldGroup {
  name: string;
  label?: string;
  fields: string[];
  collapsed?: boolean;
}

export interface FrontmatterConfig {
  version: string;
  previewImageField?: string;
  customFields: FrontmatterFieldConfig[];
  fieldGroups: FrontmatterFieldGroup[];
}

export interface Page {
  id: string;
  title: string;
  content: string;
  frontmatter: Frontmatter;
  filePath: string;
  createdAt: number;
  modifiedAt: number;
}

export interface Draft {
  id: string;
  title: string;
  content: string;
  frontmatter: Frontmatter;
  filePath: string;
  createdAt: number;
  modifiedAt: number;
}

export interface ImageInfo {
  filename: string;
  path: string;
  fullPath: string;
  url: string;
  size: number;
  width?: number;
  height?: number;
  createdAt: number;
}

export interface HexoConfig {
  title: string;
  subtitle: string;
  description: string;
  author: string;
  language: string;
  url: string;
}

export interface AppConfig {
  version: string;
  lastProjectPath?: string;
  recentProjects: string[];
  uiLanguage: string;
  theme: 'light' | 'dark' | 'auto';
  autoSaveEnabled: boolean;
  autoSaveInterval: number;
  editorFontSize: number;
  editorLineHeight: number;
}

export interface CommandOutput {
  success: boolean;
  stdout: string;
  stderr: string;
  exitCode: number;
}
