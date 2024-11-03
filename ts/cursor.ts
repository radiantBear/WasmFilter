export function saveCursorPosition(element: HTMLElement | null): number | null {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0 || !element) return null;

    const range = selection.getRangeAt(0);
    const preCursorRange = range.cloneRange();
    preCursorRange.selectNodeContents(element);
    preCursorRange.setEnd(range.startContainer, range.startOffset);

    return preCursorRange.toString().length;  // Character offset
}

export function restoreCursorPosition(element: HTMLElement | null, cursorPosition: number | null) {
    if (element === null || cursorPosition === null) return;

    const selection = window.getSelection();
    if (!selection) return;

    let chars = cursorPosition;
    const range = document.createRange();

    // Find where to set the cursor within `element`'s nested contents
    function setCursorToPosition(node: any) {
        if (chars === 0) return;

        if (node.nodeType === Node.TEXT_NODE) {
            const textLength = node.textContent?.length || 0;
            if (textLength >= chars) {
                range.setStart(node, chars);
                chars = 0;
            } else {
                chars -= textLength;
            }
        } else {
            for (const child of node.childNodes) {
                setCursorToPosition(child);
                if (chars === 0) break;
            }
        }
    }

    setCursorToPosition(element);
    // Set to be single cursor, not highlighted selection
    range.collapse(true);

    selection.removeAllRanges();
    selection.addRange(range);
}