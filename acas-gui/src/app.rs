use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use yew::virtual_dom::{VNode, VText};
use yew::{function_component, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub html: String,
}

#[function_component(SafeHtml)]
pub fn safe_html(props: &Props) -> Html {
    let div = gloo_utils::document().create_element("div").unwrap();
    div.set_inner_html(&props.html.clone());

    Html::VRef(div.into())
}

#[derive(Serialize, Deserialize)]
struct ParseArgs<'a> {
    expression: &'a str,
}

#[function_component(App)]
pub fn app() -> Html {
    let math_input_ref = use_node_ref();

    let expression = use_state(|| String::new());

    let latex_output = use_state(|| VNode::VText(VText::new("")));
    {
        let latex_output = latex_output.clone();
        let expression = expression.clone();
        let expression2 = expression.clone();
        use_effect_with_deps(
            move |_| {
                spawn_local(async move {
                    if expression.is_empty() {
                        return;
                    }

                    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
                    let new_output = invoke(
                        "parse",
                        to_value(&ParseArgs { expression: &*expression }).unwrap(),
                    )
                    .await;
                    let s = new_output.as_string().unwrap();
                    log(&s);

                    let rendered = katex::render(&s).unwrap();
                    latex_output.set( Html::from_html_unchecked(rendered.into()));
                });

                || {}
            },
            expression2,
        );
    }

    let math = {
        let expression = expression.clone();
        let math_input_ref = math_input_ref.clone();
        Callback::from(move |_| {
            expression.set(math_input_ref.cast::<web_sys::HtmlInputElement>().unwrap().value());
        })
    };

    html! {
        <main class="container">
            <div class="row">
                <textarea id="math-input" ref={math_input_ref} placeholder="Enter math" />
                <button type="button" onclick={math}>{"Parse"}</button>
            </div>

            <div>{ (&*latex_output).clone() }</div>
        </main>
    }
}
