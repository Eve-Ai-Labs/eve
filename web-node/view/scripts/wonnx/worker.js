import {
  env,
  AutoTokenizer,
  AutoModelForCausalLM,
  TextStreamer,
  InterruptableStoppingCriteria,
} from "/scripts/transformers/transformers.min.js";

const WONNX_LOADING = "wonnx.loading";
const WONNX_LOADING_STARTED = "wonnx.started";
const WONNX_LOADING_PROGRESS = "wonnx.progress";
const WONNX_LOADING_COMPLETE = "wonnx.complete";
const WONNX_LOADING_COMPILE = "wonnx.compile";

const WONNX_GENERATE = "wonnx.generate";
const WONNX_GENERATE_STARTED = "wonnx.started";
const WONNX_GENERATE_PROGRESS = "wonnx.progress";
const WONNX_GENERATE_COMPLETE = "wonnx.complete";

const stopping_criteria = new InterruptableStoppingCriteria();
let past_key_values_cache = null;
env.backends.onnx.wasm.wasmPaths = "/scripts/transformers/";

class TextGenerationPipeline {
  static model_id = "onnx-community/DeepSeek-R1-Distill-Qwen-1.5B-ONNX";

  static async getInstance(progress_callback = null) {
    let device = null,
      dtype = "int8";
    if (navigator.gpu) {
      device = "webgpu";
      if ((await navigator.gpu.requestAdapter()).features.has("shader-f16")) {
        dtype = "q4f16";
      }
    }
    console.log("Use dtype: ", dtype);

    this.tokenizer ??= AutoTokenizer.from_pretrained(this.model_id, {
      progress_callback,
    });
    this.model ??= AutoModelForCausalLM.from_pretrained(this.model_id, {
      dtype,
      device,
      progress_callback,
    });

    return Promise.all([this.tokenizer, this.model]);
  }
}

async function generate(messages) {
  const [tokenizer, model] = await TextGenerationPipeline.getInstance();

  const inputs = tokenizer.apply_chat_template(messages, {
    add_generation_prompt: true,
    return_dict: true,
  });

  // 151648: <think>
  // 151649: </think>
  const [START_THINKING_TOKEN_ID, END_THINKING_TOKEN_ID] = tokenizer.encode(
    "<think></think>",
    { add_special_tokens: false }
  );

  let state = "thinking"; // 'thinking' or 'answering'
  let startTime;
  let numTokens = 0;
  let tps;
  const token_callback_function = (tokens) => {
    startTime ??= performance.now();

    if (numTokens++ > 0) {
      tps = (numTokens / (performance.now() - startTime)) * 1000;
    }
    if (tokens[0] == END_THINKING_TOKEN_ID) {
      state = "answering";
    }
  };
  const callback_function = (output) => {
    self.postMessage({
      tp: WONNX_GENERATE,
      progress_type: WONNX_GENERATE_PROGRESS,
      output,
      tps,
      numTokens,
      state,
    });
  };

  const streamer = new TextStreamer(tokenizer, {
    skip_prompt: true,
    skip_special_tokens: true,
    callback_function,
    token_callback_function,
  });

  // Tell the main thread we are starting
  self.postMessage({
    tp: WONNX_GENERATE,
    progress_type: WONNX_GENERATE_STARTED,
  });

  const { past_key_values, sequences } = await model.generate({
    ...inputs,
    // TODO: Add back when fixed
    // past_key_values: past_key_values_cache,

    // Sampling
    do_sample: false,
    // repetition_penalty: 1.1,
    // top_k: 3,
    // temperature: 0.2,

    max_new_tokens: 2048,
    streamer,
    stopping_criteria,
    return_dict_in_generate: true,
  });
  past_key_values_cache = past_key_values;

  const decoded = tokenizer.batch_decode(sequences, {
    skip_special_tokens: true,
  });

  // Send the output back to the main thread
  self.postMessage({
    tp: WONNX_GENERATE,
    progress_type: WONNX_GENERATE_COMPLETE,
    output: decoded,
  });
}

async function load() {
  self.postMessage({
    tp: WONNX_LOADING,
    progress_type: WONNX_LOADING_STARTED,
    data: "Loading model...",
  });

  const [tokenizer, model] = await TextGenerationPipeline.getInstance((x) => {
    x.tp = WONNX_LOADING;
    x.progress_type = WONNX_LOADING_PROGRESS;
    self.postMessage(x);
  });

  self.postMessage({
    tp: WONNX_LOADING,
    progress_type: WONNX_LOADING_COMPILE,
    data: "Compiling shaders and warming up model...",
  });

  const inputs = tokenizer("a");
  await model.generate({ ...inputs, max_new_tokens: 1 });
  self.postMessage({
    tp: WONNX_LOADING,
    progress_type: WONNX_LOADING_COMPLETE,
  });
}

// Listen for messages from the main thread
self.addEventListener("message", async (e) => {
  const { type, data } = e.data;

  switch (type) {
    case "load":
      load();
      break;

    case "generate":
      stopping_criteria.reset();
      generate(data);
      break;

    case "interrupt":
      stopping_criteria.interrupt();
      break;

    case "reset":
      past_key_values_cache = null;
      stopping_criteria.reset();
      break;
  }
});
