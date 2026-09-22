/* Glance preview bridge: render markdown + KaTeX, outline anchors */

function slugify(text) {
  return String(text)
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "sec";
}

function renderMarkdown(md, outlineJson) {
  const content = document.getElementById("content");
  if (!content) return "[]";

  let html = "";
  try {
    if (typeof marked !== "undefined" && marked.parse) {
      html = marked.parse(md || "", { gfm: true, breaks: false });
    } else {
      content.textContent = md || "";
      return "[]";
    }
  } catch (e) {
    content.textContent = "渲染失败: " + e + "\n\n" + (md || "");
    return "[]";
  }

  content.innerHTML = html;

  let outline = [];
  try {
    outline = outlineJson ? JSON.parse(outlineJson) : [];
    if (!Array.isArray(outline)) outline = [];
  } catch (_) {
    outline = [];
  }

  const heads = content.querySelectorAll("h1,h2,h3,h4,h5,h6");
  heads.forEach((el, i) => {
    const meta = outline[i];
    const id = (meta && meta.id) || `${slugify(el.textContent)}-${i}`;
    el.id = id;
    el.dataset.level = String(el.tagName).replace("H", "");
    if (meta && meta.level) el.dataset.level = String(meta.level);
  });

  if (typeof renderMathInElement === "function") {
    try {
      renderMathInElement(content, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "$", right: "$", display: false },
          { left: "\\[", right: "\\]", display: true },
          { left: "\\(", right: "\\)", display: false },
        ],
        throwOnError: false,
        errorColor: "#a15c5c",
        ignoredTags: ["script", "noscript", "style", "textarea", "pre", "code"],
      });
    } catch (e) {
      console.error("KaTeX error", e);
    }
  }

  return JSON.stringify(
    Array.from(heads).map((el) => ({
      id: el.id,
      level: Number(el.dataset.level || 1),
      text: el.textContent.trim(),
    }))
  );
}

function scrollToHeading(id) {
  const el = document.getElementById(id);
  if (el) {
    el.scrollIntoView({ behavior: "smooth", block: "start" });
    return true;
  }
  return false;
}

function flushPending() {
  if (typeof window.__pending_md === "string") {
    const md = window.__pending_md;
    const outline = window.__pending_outline || "[]";
    window.__pending_md = undefined;
    window.__pending_outline = undefined;
    renderMarkdown(md, outline);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  flushPending();
  const params = new URLSearchParams(location.search);
  const md = params.get("md");
  if (md !== null) {
    renderMarkdown(md, params.get("outline") || "[]");
  }
});

window.addEventListener("load", flushPending);
