export type Result<T, E> = { ok: true, value: T }
    | { ok: false, error: E};

export function Ok<T>(value: T): Result<T, never> { return { ok: true, value }}

export function Err<E>(error: E): Result<never, E> { return { ok: false, error }}

/**
 * Calls a function that may throw an error and converts the result to a `Result` instead:
 *      Ok for success and Err for failure
 * @param fn The function to call that may throw
 * @param args The arguments to pass to the function
 */
export function toResult<T extends (...args: any) => any>(fn: T, ...args: Parameters<T>): Result<ReturnType<T>, any> {
    try {
        return Ok( fn.call(null, ...args) );
    }
    catch (error) {
        return Err( error );
    }
}
