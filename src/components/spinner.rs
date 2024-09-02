use leptos::prelude::*;

#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div
            class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-2 overflow-clip bg-neutral-400 bg-opacity-50  dark:bg-neutral-900 dark:bg-opacity-50"
            style="perspective: 600px; backdrop-filter: blur(2px);"
        >
            <svg
                viewBox="0 0 59 59"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
                class="spinner z-[50] mt-12 flex max-h-[50vh] max-w-[320px] opacity-80"
            >
                <path
                    d="M29.5 57C44.6878 57 57 44.6878 57 29.5C57 14.3122 44.6878 2 29.5 2C14.3122 2 2 14.3122 2 29.5C2 44.6878 14.3122 57 29.5 57Z"
                    fill="none"
                    class="stroke-[#75b6e7] dark:stroke-[#75b6e7]"
                    stroke-width="2.0122"
                    stroke-miterlimit="10"
                />
                <path
                    d="M20.3672 18.8621L26.0882 15.8511L24.5827 22.1743L35.1214 28.3094L40.8425 33.6164L40.2403 40.2408L36.0248 42.3485L35.1214 36.3264L24.2816 30.3042L19.4639 24.5832L20.3672 18.8621Z"
                    fill="none"
                />
                <path
                    d="M28.9569 44.8452C31.7683 44.9192 36.2073 43.4395 36.2813 40.1102C36.5772 30.6403 19.0431 33.3777 19.413 22.4281C19.561 17.4712 25.5536 14.2899 30.5105 14.5119"
                    class="stroke-[#75b6e7] dark:stroke-[#75b6e7]"
                    stroke-width="2.0122"
                    stroke-miterlimit="10"
                />
                <path
                    d="M41.2382 19.8387C38.7227 16.8794 35.8374 14.8078 31.6203 14.5119C28.8089 14.29 24.8138 15.6217 24.5179 18.9509C23.704 28.2729 42.126 26.5712 41.1642 37.5208C40.7203 42.4777 34.1357 44.9932 28.9569 44.8452C24.7398 44.6972 21.1146 42.7737 18.1553 39.5184"
                    class="stroke-[#75b6e7] dark:stroke-[#75b6e7]"
                    stroke-width="2.0122"
                    stroke-miterlimit="10"
                />
            </svg>
            <code class="unstyled rocker flex items-center whitespace-nowrap rounded bg-surface-50 px-2 py-0.5 font-mono text-lg font-bold dark:bg-[#28292a]">
                Loading...
            </code>
        </div>
    }
}
