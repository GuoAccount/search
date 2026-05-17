import { ScanResult, TreeNode } from "../types";
import { DEFAULT_PRESETS } from "../constants/presets";

export function getFileCategory(extension: string): string {
  const ext = extension.toLowerCase();
  for (const [key, preset] of Object.entries(DEFAULT_PRESETS)) {
    if (preset.extensions.includes(ext)) {
      return key;
    }
  }
  return "other";
}

export function getFileIcon(extension: string): string {
  const category = getFileCategory(extension);
  switch (category) {
    case "document":
      return "FileText";
    case "code":
      return "FileCode2";
    case "image":
      return "Image";
    case "config":
      return "Settings";
    default:
      return "File";
  }
}

export function getMatchTypeLabel(matchType: string): string {
  switch (matchType) {
    case "filename":
      return "文件名";
    case "content":
      return "内容";
    case "exif":
      return "EXIF";
    case "ocr":
      return "OCR";
    default:
      return matchType;
  }
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export function getFileDir(filePath: string): string {
  const parts = filePath.split("/");
  parts.pop();
  return parts.join("/") || "/";
}

export function buildTree(results: ScanResult[]): TreeNode {
  const root: TreeNode = {
    name: "root",
    path: "",
    isDir: true,
    children: [],
  };

  for (const result of results) {
    const parts = result.file_path.split("/").filter(Boolean);
    let current = root;

    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const path = "/" + parts.slice(0, i + 1).join("/");
      const isLast = i === parts.length - 1;

      let child = current.children.find((c) => c.path === path);
      if (!child) {
        child = {
          name: part,
          path,
          isDir: !isLast,
          children: [],
          result: isLast ? result : undefined,
        };
        current.children.push(child);
      }

      if (!isLast) {
        current = child;
      }
    }
  }

  // Sort children: directories first, then files
  const sortChildren = (node: TreeNode) => {
    node.children.sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
    node.children.forEach(sortChildren);
  };

  sortChildren(root);
  return root;
}

export function getAllExtensions(enabledPresets: string[], customExtensions: Record<string, string[]>): string[] {
  const exts: string[] = [];
  for (const preset of enabledPresets) {
    const defaultExts = DEFAULT_PRESETS[preset]?.extensions || [];
    const custom = customExtensions[preset] || [];
    exts.push(...defaultExts, ...custom);
  }
  return [...new Set(exts)].map((e) => e.toLowerCase());
}

export function getTabCounts(results: ScanResult[]) {
  const counts = {
    all: results.length,
    document: 0,
    code: 0,
    image: 0,
    config: 0,
  };

  for (const result of results) {
    const category = getFileCategory(result.file_extension);
    if (category in counts) {
      counts[category as keyof typeof counts]++;
    }
  }

  return counts;
}
