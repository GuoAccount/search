import { FileText, FileCode2, Image, Settings } from "lucide-react";
import { FileTypePreset } from "../types";

export const STORAGE_KEY = "lumina_settings";

export const DEFAULT_PRESETS: Record<string, FileTypePreset> = {
  document: {
    label: "文档",
    icon: FileText,
    extensions: ["txt", "md", "csv", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"],
  },
  code: {
    label: "代码",
    icon: FileCode2,
    extensions: ["rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "hpp", "rb", "php", "swift", "kt", "scala", "sh", "bash", "zsh", "fish", "sql", "graphql", "proto"],
  },
  image: {
    label: "图片",
    icon: Image,
    extensions: ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "svg", "ico"],
  },
  config: {
    label: "配置",
    icon: Settings,
    extensions: ["env", "gitignore", "dockerignore", "makefile", "cmake", "ini", "cfg", "conf", "config", "json", "yaml", "yml", "toml", "xml"],
  },
};

export const DEFAULT_SETTINGS = {
  scanPath: "",
  keyword: "",
  enabledPresets: ["document", "code", "config"],
  customExtensions: {},
  ocrEnabled: false,
  sidebarOpen: true,
  theme: "system" as const,
};

export const SCAN_TYPES = ["file_name", "text_content"] as const;

export const IMAGE_EXTENSIONS = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "heic", "heif", "svg", "ico"];

export const TEXT_EXTENSIONS = ["txt", "md", "csv", "json", "xml", "yaml", "yml", "toml", "rs", "py", "js", "ts", "tsx", "jsx", "go", "java", "c", "cpp", "h", "html", "css", "scss", "less", "sh", "bash", "zsh", "fish", "sql", "graphql", "proto", "ini", "cfg", "conf", "config"];
