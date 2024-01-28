use leptos::{html::Input, *};


#[component]
pub fn TypeArea(send: Action<String, Result<String, ServerFnError>>) -> impl IntoView {
    let input_ref = create_node_ref::<Input>();
    view! {
        <div class="absolute bottom-0 h-10 w-full">
            <form on:submit=move |ev| {
                ev.prevent_default();
                let input = input_ref.get().expect("input to exist");
                send.dispatch(input.value());
                input.set_value("");
            }>
                <input type="text" node_ref=input_ref class="w-2/3 focus:border-solid focus:border-blue-500 border-gray-200 border-2"/>
                <input type="submit" class="w-1/3"/>
            </form>
        </div>
    }
}