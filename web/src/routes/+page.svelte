<script>
    import "../app.css";
    
        function handleSubmit() {
        // prevent form from submitting traditionally
        event.preventDefault();
        alert("Button was clicked!");
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
        <form class="w-full">
            <div class="mb-4 border border-black border-2 rounded-lg h-fit">
                <div class="p-4 bg-white rounded-t-lg">
                    <textarea
                        id="comment"
                        rows={4}
                        class="w-full px-8 py-6 text-xl border-black focus:shadow-soft-primary-outline appearance-none rounded-lg border-2 border-solid"
                        placeholder="monkey code goes here"
                    />
                </div>
                    <div
                        class="flex items-center justify-between px-3 py-2 border-black border-t-2 space-x-5"
                    >
                        <div class="flex"></div>
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
</div>
<!-- This is where the content of any nested route will be rendered -->
<slot />
