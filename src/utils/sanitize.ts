import MarkdownIt from "markdown-it";
import DOMPurify from "dompurify";

const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: false,
});

// 禁用远程图片自动加载
const defaultImageRender = md.renderer.rules.image;
md.renderer.rules.image = (tokens, idx, options, env, self) => {
  const token = tokens[idx];
  const src = token.attrGet("src") || "";
  // 阻止远程图片
  if (src.startsWith("http://") || src.startsWith("https://")) {
    token.attrSet("src", "");
    token.attrSet("data-blocked-src", src);
  }
  if (defaultImageRender) {
    return defaultImageRender(tokens, idx, options, env, self);
  }
  return self.renderToken(tokens, idx, options);
};

// 禁用 javascript: 链接
const defaultLinkRender = md.renderer.rules.link_open;
md.renderer.rules.link_open = (tokens, idx, options, env, self) => {
  const token = tokens[idx];
  const href = token.attrGet("href") || "";
  if (href.startsWith("javascript:")) {
    token.attrSet("href", "#");
  }
  if (defaultLinkRender) {
    return defaultLinkRender(tokens, idx, options, env, self);
  }
  return self.renderToken(tokens, idx, options);
};

export function renderMarkdown(content: string): string {
  const rawHtml = md.render(content);
  return DOMPurify.sanitize(rawHtml, {
    ALLOWED_TAGS: [
      "h1", "h2", "h3", "h4", "h5", "h6",
      "p", "br", "hr",
      "ul", "ol", "li",
      "code", "pre", "blockquote",
      "a", "img",
      "table", "thead", "tbody", "tr", "th", "td",
      "strong", "em", "del", "s",
      "div", "span",
    ],
    ALLOWED_ATTR: [
      "href", "src", "alt", "title",
      "class", "id",
      "data-blocked-src",
    ],
    FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover", "onfocus"],
    FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "form"],
  });
}

export function sanitizeSvg(svgContent: string): string {
  return DOMPurify.sanitize(svgContent, {
    USE_PROFILES: { svg: true, svgFilters: true },
    FORBID_TAGS: ["script", "foreignObject"],
    FORBID_ATTR: ["onload", "onerror", "onclick"],
  });
}
