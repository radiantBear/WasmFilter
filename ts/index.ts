import * as wasm from "../pkg";
import {BareToken} from "../pkg";

wasm.greet();

type Result<T, E> = { ok: true, value: T }
                  | { ok: false, error: E};
function Ok<T>(value: T): Result<T, never> { return { ok: true, value }}
function Err<E>(error: E): Result<never, E> { return { ok: false, error }}
function toResult<T extends (...args: any) => any>(fn: T, ...args: Parameters<T>): Result<ReturnType<T>, any> {
    try {
        return Ok( fn.call(null, ...args) );
    }
    catch (error) {
        return Err( error );
    }
}

function handleFilterSubmit(e: SubmitEvent) {
    const start = performance.now();
    e.preventDefault();

    const button =  document.getElementById('filter-input-submit');
    if (button)
        // @ts-ignore
        button.disabled = true

    const input = document?.getElementById('filter-input')?.textContent;
    wasm.parse_filter(input ?? '');

    if (button)
        // @ts-ignore
        button.disabled = false

    const end = performance.now();
    console.log(`Parsing took ${end - start} ms`);
}

function handleFilterKeydown(e: KeyboardEvent) {
    if (e.shiftKey && e.key === 'Enter') {
        e.preventDefault();

        const inputElement = document.getElementById('filter-input');
        if (!inputElement) return;

        let position = saveCursorPosition(inputElement);

        const selection = window.getSelection();
        const range = selection?.getRangeAt(0);
        if (!range)
            return;
        if (!inputElement.contains(range.endContainer))
            // Prevent weird error from user pressing Delete then Enter on empty field
            return;

        range.deleteContents();
        // For some reason, second newline needed if cursor is at the end of all input contents
        if (range.startContainer.parentElement?.lastChild
            && range.intersectsNode(range.startContainer.parentElement?.lastChild)
            && range.endOffset === range.startContainer.textContent?.length
        )
            range.insertNode(document.createTextNode('\n'));
        range.insertNode(document.createTextNode('\n'));
        range.collapse(false);

        if (position)
            restoreCursorPosition(inputElement, position + 1);
    }
    else if (e.key === 'Enter') {
        e.preventDefault();     // Don't add a newline
        handleFilterSubmit(new SubmitEvent('submit'));
    }
}

function handleFilterInput(e: Event) {
    e.preventDefault();

    const inputElement = document.getElementById('filter-input');
    if (!inputElement) return;

    const position = saveCursorPosition(inputElement);
    const input = inputElement.textContent ?? '';

    // Tokenize input
    const output = toResult(wasm.lex_filter, input);

    if (!output.ok) {
        // Display the error message to the user
        const filter_error = document.getElementById('filter-error');
        if (filter_error) {
            filter_error.textContent = output.error;
            filter_error.classList.remove('d-none');
        }
        return;
    }
    else {
        // Ensure no error message is present
        const filter_error = document.getElementById('filter-error');
        if (filter_error) {
            filter_error.textContent = '';
            filter_error.classList.add('d-none');
        }
    }

    // Generate syntax-highlighted HTML
    const wrapper = document.createElement('span');
    let last_end = 0;
    for (const token of output.value) {
        let className = '';
        switch (token.token) {
            case BareToken.Name:
                className = 'hl-name';
                break;
            case BareToken.Paren:
            case BareToken.Comparator:
                className = 'hl-cmpr';
                break;
            case BareToken.String:
                className = 'hl-str';
                break;
            case BareToken.Number:
                className = 'hl-num';
                break;
            case BareToken.JoinType:
                className = 'hl-join';
                break;
        }

        // Append non-highlighted text
        const textSegment = document.createTextNode(input.slice(last_end, token.start));
        last_end = token.end;

        // Append highlighted text
        const highlightedSegment = document.createElement('span');
        highlightedSegment.className = className;
        highlightedSegment.textContent = input.slice(token.start, token.end);

        wrapper.append(textSegment, highlightedSegment);
    }

    // Append remaining whitespace
    wrapper.append(document.createTextNode(input.slice(last_end)));

    // Replace the input with the highlighted version
    inputElement.innerHTML = wrapper.innerHTML;
    restoreCursorPosition(inputElement, position);
}

document?.getElementById('filter-input-form')
    ?.addEventListener('submit', handleFilterSubmit);

document?.getElementById('filter-input')
    ?.addEventListener('keydown', handleFilterKeydown);

document.getElementById('filter-input')
    ?.addEventListener('input', handleFilterInput);

function saveCursorPosition(element: HTMLElement | null): number | null {
    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0 || !element) return null;

    const range = selection.getRangeAt(0);
    const preCursorRange = range.cloneRange();
    preCursorRange.selectNodeContents(element);
    preCursorRange.setEnd(range.startContainer, range.startOffset);

    return preCursorRange.toString().length;  // Character offset
}

function restoreCursorPosition(element: HTMLElement | null, cursorPosition: number | null) {
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