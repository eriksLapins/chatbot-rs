use leptos::*;
use cfg_if::cfg_if;
use crate::model::conversation::Conversation;


// no security atm;
#[server(Converse, "/api")]
pub async fn converse(prompt: Conversation) -> Result<String, ServerFnError> {
    use llm::models::Llama;
    use leptos_actix::extract;
    use actix_web::web::Data;

    let model = extract::<Data::<Llama>>().await.unwrap().into_inner();

    use llm::KnownModel;
    let character_name = "### Assistant";
    let user_name = "### Human";
    let persona = "A chat between a human and an assistant";
    let mut history = format!(
        "{character_name}:Hello - How may I help you today?\n\
        {user_name}:What is the capital of France?\n\
        {character_name}:Paris is the capital of France.\n"
    );

    for message in prompt.messages.into_iter() {
        let msg = message.text;
        
        let curr_line = if message.user {
            format!("{character_name}: {msg}\n")
        } else {
            format!("{user_name}: {msg}\n")
        };

        history.push_str(&curr_line);
    }

    let mut res = String::new();
    let mut rng = rand::thread_rng();
    let mut buf = String::new();

    let mut session = model.start_session(Default::default());

    println!("before infer");

    session.infer(
        model.as_ref(),
        &mut rng,
        &llm::InferenceRequest {
            prompt: format!("{persona}\n{history}\n{character_name}:")
                .as_str()
                .into(),
            parameters: &llm::InferenceParameters::default(),
            play_back_previous_tokens: false,
            maximum_token_count: None,
        },
        &mut Default::default(),
        inference_callback(String::from(user_name), &mut buf, &mut res),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    println!("got through infer");

    Ok(res)
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use std::convert::Infallible;
            fn inference_callback<'a>(
                stop_sequence: String,
                buf: &'a mut String,
                out_str: &'a mut String,
            ) -> impl FnMut(llm::InferenceResponse) -> Result<llm::InferenceFeedback, Infallible> + 'a {
                use llm::InferenceFeedback::{Halt, Continue};

                move |resp| -> Result<llm::InferenceFeedback, Infallible> {match resp {
                    llm::InferenceResponse::InferredToken(t) => {
                        println!("got to token");
                        let mut reverse_buf = buf.clone();
                        println!("got here");
                        reverse_buf.push_str(t.as_str());
                        if stop_sequence.as_str().eq(reverse_buf.as_str()) {
                            buf.clear();
                            return Ok::<llm::InferenceFeedback, Infallible>(Halt);
                        } else if stop_sequence.as_str().starts_with(reverse_buf.as_str()) {
                            buf.push_str(t.as_str());
                            return Ok(Continue);
                        }

                        if buf.is_empty() {
                            out_str.push_str(&t);
                        } else {
                            out_str.push_str(&reverse_buf);
                        }

                        Ok(Continue)
                    },
                    llm::InferenceResponse::EotToken => {
                        println!("eot token");
                        Ok(Halt)
                    },
                    _ => {
                        println!("matched any");
                        Ok(Continue)},
                }}
            }
    }
}