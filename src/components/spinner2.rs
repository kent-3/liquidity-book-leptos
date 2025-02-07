use leptos::prelude::*;

#[component]
pub fn Spinner2(#[prop(optional, into)] size: String) -> impl IntoView {
    let defaults =
        "ring-1 ring-[#ff3912] rounded-full p-[4px] animate-spin fill-[#ff3912]".to_string();
    // let class = format!("{defaults} {size}");

    view! {
        <svg
            class=format!("{defaults} {size}")
            xmlns="http://www.w3.org/2000/svg"
            version="1.1"
            viewBox="0 0 308.156 312.193"
        >
            <path d="M308.156,94.509c0-4.797-2.467-9.127-6.588-11.583L169.465,4.246c-9.492-5.662-21.283-5.662-30.775,0L6.588,82.927C2.514,85.355.067,89.613.001,94.345c.002,5.785.005,15.249.006,21.035-.165,4.982,2.326,9.514,6.58,12.049l46.44,27.669c.76.453.76,1.554,0,2.006l-46.44,27.659C2.474,187.216.008,191.535,0,196.323c0,5.872,0,15.479,0,21.351,0,4.808,2.467,9.137,6.588,11.583l132.102,78.691c4.746,2.831,10.064,4.246,15.393,4.246s10.636-1.415,15.382-4.246l132.102-78.691c4.121-2.446,6.588-6.775,6.588-11.583,0-5.872,0-15.479,0-21.351-.008-4.788-2.474-9.107-6.588-11.559l-46.44-27.659c-.76-.453-.76-1.554,0-2.006l46.44-27.669c4.121-2.456,6.588-6.785,6.588-11.583v-21.337ZM144.935,137.889c2.82-1.675,5.984-2.519,9.148-2.519s6.317.843,9.138,2.519l30.567,18.213-30.567,18.203c-5.641,3.362-12.645,3.362-18.286,0l-30.567-18.203,30.567-18.213ZM277.58,206.012c.76.453.76,1.554,0,2.007l-114.359,68.113c-5.641,3.362-12.645,3.362-18.286,0L30.575,208.018c-.76-.453-.76-1.554,0-2.006l21.431-12.769c4.188-2.496,6.754-7.011,6.754-11.887v-20.078c0-1.182,1.287-1.914,2.303-1.31l77.627,46.151c4.746,2.82,10.064,4.236,15.393,4.236s10.636-1.416,15.382-4.236l22.651-13.495c4.214-2.511,6.784-7.065,6.755-11.97l-.113-18.898c-.007-1.186,1.284-1.926,2.304-1.318l76.518,45.573ZM249.145,150.749c0,1.182-1.287,1.914-2.303,1.31l-77.377-45.986c-9.492-5.651-21.283-5.651-30.775,0l-22.796,13.578c-4.215,2.511-6.785,7.065-6.756,11.971l.117,19.341c.005.909-.984,1.475-1.765,1.01L30.576,106.181c-.76-.453-.761-1.554,0-2.007l114.359-68.113c2.82-1.686,5.984-2.529,9.148-2.529s6.317.843,9.138,2.529l114.359,68.113c.76.453.76,1.554,0,2.007l-21.68,12.915c-4.189,2.495-6.755,7.011-6.755,11.887v19.765Z" />
        </svg>
    }

    // view! {
    //     <svg
    //         class=format!("{defaults} {size}")
    //         fill="none"
    //         stroke="black"
    //         stroke-width="2.0122"
    //         stroke-miterlimit="10"
    //         viewBox="0 0 59 59"
    //         xmlns="http://www.w3.org/2000/svg"
    //     >
    //         <path d="M29.5 57C44.6878 57 57 44.6878 57 29.5C57 14.3122 44.6878 2 29.5 2C14.3122 2 2 14.3122 2 29.5C2 44.6878 14.3122 57 29.5 57Z" />
    //         <path d="M20.3672 18.8621L26.0882 15.8511L24.5827 22.1743L35.1214 28.3094L40.8425 33.6164L40.2403 40.2408L36.0248 42.3485L35.1214 36.3264L24.2816 30.3042L19.4639 24.5832L20.3672 18.8621Z" />
    //         <path d="M28.9569 44.8452C31.7683 44.9192 36.2073 43.4395 36.2813 40.1102C36.5772 30.6403 19.0431 33.3777 19.413 22.4281C19.561 17.4712 25.5536 14.2899 30.5105 14.5119" />
    //         <path d="M41.2382 19.8387C38.7227 16.8794 35.8374 14.8078 31.6203 14.5119C28.8089 14.29 24.8138 15.6217 24.5179 18.9509C23.704 28.2729 42.126 26.5712 41.1642 37.5208C40.7203 42.4777 34.1357 44.9932 28.9569 44.8452C24.7398 44.6972 21.1146 42.7737 18.1553 39.5184" />
    //     </svg>
    // }
}
