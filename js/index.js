import * as wasm from "../pkg/index";

wasm.greet();

let input = prompt("Query: ");
while (input !== 'stop') {
    wasm.parse_filter(input)

    input = prompt("Query: ");
}