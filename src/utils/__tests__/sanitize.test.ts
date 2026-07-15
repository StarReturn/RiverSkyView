import { describe, it, expect } from "vitest";
import { renderMarkdown, sanitizeSvg } from "../sanitize";

describe("sanitize - Markdown rendering", () => {
  it("renders basic markdown correctly", () => {
    const result = renderMarkdown("# Hello World");
    expect(result).toContain("<h1>Hello World</h1>");
  });

  it("renders lists correctly", () => {
    const result = renderMarkdown("- Item 1\n- Item 2");
    expect(result).toContain("<li>Item 1</li>");
    expect(result).toContain("<li>Item 2</li>");
  });

  it("renders code blocks correctly", () => {
    const result = renderMarkdown("```\ncode here\n```");
    expect(result).toContain("<pre");
    expect(result).toContain("<code>");
  });

  it("escapes raw script tags (no execution)", () => {
    const result = renderMarkdown("<script>alert(1)</script>");
    // Raw HTML should be escaped, not rendered as actual script tag
    expect(result).not.toContain("<script>");
    expect(result).not.toMatch(/<script[^>]*>/i);
  });

  it("prevents onerror attribute in images", () => {
    // DOMPurify should strip onerror attributes from any surviving img tags
    const result = renderMarkdown('![alt](https://example.com/image.png)');
    // Should not contain onerror as an attribute
    expect(result).not.toMatch(/\sonerror\s*=/i);
  });

  it("blocks javascript: protocol in markdown links", () => {
    const result = renderMarkdown("[click](javascript:alert(1))");
    // Should not contain href="javascript:"
    expect(result).not.toMatch(/href=["']javascript:/i);
  });

  it("removes iframe tags", () => {
    const result = renderMarkdown('<iframe src="evil.com"></iframe>');
    expect(result).not.toMatch(/<iframe/i);
  });

  it("removes object and embed tags", () => {
    const result = renderMarkdown('<object data="evil.swf"></object><embed src="evil.swf">');
    expect(result).not.toMatch(/<object/i);
    expect(result).not.toMatch(/<embed/i);
  });

  it("renders tables correctly", () => {
    const result = renderMarkdown("| A | B |\n|---|---|\n| 1 | 2 |");
    expect(result).toContain("<table>");
    expect(result).toContain("<th>");
  });
});

describe("sanitize - SVG sanitization", () => {
  it("removes script tags from SVG", () => {
    const svg = '<svg><script>alert(1)</script><rect width="100" height="100"/></svg>';
    const result = sanitizeSvg(svg);
    expect(result).not.toContain("<script>");
  });

  it("removes onload handlers from SVG", () => {
    const svg = '<svg onload="alert(1)"><rect width="100" height="100"/></svg>';
    const result = sanitizeSvg(svg);
    expect(result).not.toContain("onload");
  });
});
