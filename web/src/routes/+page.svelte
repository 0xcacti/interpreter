<script lang="ts">
    import "../app.css";

    let wasmModule: any;
    let wasmReady = false;
    let result: string | null = null;

    // @ts-ignore
    import('/pkg/interpreter.js?url').then((module: any) => {
        wasmModule = module;
        console.log("Wasm module loaded:", wasmModule);

        // If the module provides a readiness promise
        if (wasmModule && wasmModule.ready) {
            wasmModule.ready.then(() => {
                wasmReady = true;
            });
        } else {
            wasmReady = true;
        }
    });

    function handleSubmit(event: Event): void {
        event.preventDefault();

        if (wasmReady && wasmModule) {
            // Get the value from the textarea
            const code = (document.getElementById("monkey-code") as HTMLTextAreaElement).value;

            // Now, call the interpret function from the wasmModule directly
            result = wasmModule.exports?.interpret(code);
            console.log("Result:", result);
        } else {
            console.log("Wasm module is not yet ready.");
        }
    }
</script>

<svelte:head>
    <title>Monkey Playground</title>
</svelte:head>

<div class="bg-white m-10">
    <header class="flex w-full mb-5">
        <h1 class="text-4xl">Monkey Playground</h1>
    </header>

    <div class="flex w-full">
        <form class="w-full" on:submit={handleSubmit}>
            <div class="mb-4 border border-black border-2 rounded-lg h-fit">
                <div class="p-4 bg-white rounded-t-lg">
                    <textarea
                        id="monkey-code"
                        rows={4}
                        class="w-full px-8 py-6 text-xl border-black focus:shadow-soft-primary-outline appearance-none rounded-lg border-2 border-solid"
                        placeholder="monkey code goes here"
                    />
                </div>
                <div
                    class="flex items-center justify-between px-3 py-2 border-black border-t-2 space-x-5"
                >
                    <div class="flex" />
                    <button
                        type="submit"
                        class="inline-flex items-center py-2.5
                            px-4 font-rounded font-bold text-center text-black rounded-lg
                            bg-green-400 hover:ring-4 hover:ring-fuchsia-300 focus:ring-4
                            focus:ring-fuchsia-300"
                    >
                        Submit
                    </button>
                </div>
            </div>
        </form>
    </div>

    {#if result}
        <div>Output: {result}</div>
    {/if}
</div>
<!-- This is where the content of any nested route will be rendered -->
<slot />

