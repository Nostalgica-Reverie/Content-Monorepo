export function codeCopy(node: HTMLElement) {
  const enhance = (block: HTMLPreElement) => {
    if (block.dataset.copyEnhanced === "true") return;
    block.dataset.copyEnhanced = "true";

    const container = document.createElement("div");
    container.className = "code-wrapper";
    const copyButton = document.createElement("button");
    copyButton.type = "button";
    copyButton.innerText = "Copy";
    copyButton.className = "copy-button";
    copyButton.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(block.innerText);
        copyButton.innerText = "Copied!";
      } catch {
        copyButton.innerText = "Copy failed";
      }
      window.setTimeout(() => (copyButton.innerText = "Copy"), 2000);
    });

    block.replaceWith(container);
    container.append(copyButton, block);
  };

  const scan = () => {
    node.querySelectorAll<HTMLPreElement>("pre").forEach(enhance);
  };

  scan();
  const observer = new MutationObserver(scan);
  observer.observe(node, { childList: true, subtree: true });

  return {
    destroy() {
      observer.disconnect();
    },
  };
}
