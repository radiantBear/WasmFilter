import * as wasm from "../pkg";
import {BareToken} from "../pkg";
import {restoreCursorPosition, saveCursorPosition} from "./cursor";

document?.getElementById('filter-input-form')
    ?.addEventListener('submit', handleFilterSubmit);

document?.getElementById('filter-input')
    ?.addEventListener('keydown', handleFilterKeydown);

document.getElementById('filter-input')
    ?.addEventListener('input', handleFilterInput);

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
    const start = performance.now();
    e.preventDefault();

    const inputElement = document.getElementById('filter-input');
    if (!inputElement) return;

    const position = saveCursorPosition(inputElement);
    const input = inputElement.textContent ?? '';

    // Tokenize input
    const output = wasm.lex_filter(input);

    if (output.errors.length) {
        // Display the error message to the user
        const filter_error = document.getElementById('filter-error');
        if (filter_error) {
            filter_error.textContent = output.errors.map((err: wasm.FilterError) => err.message).join('\n');
            filter_error.classList.remove('d-none');
        }
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
    for (const token of output.tokens) {
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
            case BareToken.Error:
                className = 'hl-invalid';
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

    const end = performance.now();
    console.log(`Parsing took ${end - start} ms`);
}