import * as wasm from "../pkg/index";

wasm.greet();

document.getElementById('filter-input-form')
    .addEventListener('submit', e => {
        e.preventDefault();
        
        const input = document.getElementById('filter-input').innerText;
        wasm.parse_filter(input);
    });
