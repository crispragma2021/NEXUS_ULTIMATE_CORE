#!/usr/bin/env python3
# ==============================================================================
# 🧬 E6 — FINE-TUNING LoRA DE QWEN 2.5 (Cerebro Especializado en Scraping)
# ==============================================================================
# Entrena una capa LoRA sobre Qwen 2.5 7B con el corpus acumulado del Cerebro
# Auto-Creciente (E2). Resultado: modelo especializado en tus dominios que
# supera a Qwen base en tus tareas de extracción, corriendo en tu GPU.
#
# Especificación: plans/nexus-epic-roadmap.md §2
#   - Base: Qwen2.5-7B-Instruct (o versión abliterated)
#   - LoRA rank r=16, alpha=32, target q/v/k/o_proj
#   - QLoRA 4-bit + batch_size=1 + gradient_accumulation=8 (8 GB VRAM)
#   - Épocas 3-5, lr 2e-4, cosine schedule
#
# Requisitos:
#   pip install "unsloth[colab-new] @ git+https://github.com/unslothai/unsloth.git"
#   pip install --no-deps "xformers<0.0.27" "trl<0.9.0" peft accelerate bitsandbytes
#
# Uso:
#   python scripts/train_lora.py \
#       --dataset data/corpus_lora.jsonl \
#       --base Qwen/Qwen2.5-7B-Instruct \
#       --output models/nexus-qwen-lora \
#       --epochs 3
# ==============================================================================

import argparse
import json
import os
import sys
from typing import List, Dict

# ── Imports de Unsloth (se importan dentro para que --help funcione sin él) ──


def load_dataset(path: str) -> List[Dict[str, str]]:
    """Carga el dataset instruct: {instruction, input, output}."""
    rows = []
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            obj = json.loads(line)
            rows.append(
                {
                    "instruction": obj.get("instruction", ""),
                    "input": obj.get("input", ""),
                    "output": obj.get("output", ""),
                }
            )
    print(f"[E6] Dataset cargado: {len(rows)} ejemplos desde {path}")
    return rows


def build_chat_dataset(rows: List[Dict[str, str]]) -> List[Dict[str, str]]:
    """Convierte a formato chat de Unsloth (sharegpt: convo/gpt)."""
    out = []
    for r in rows:
        user_text = r["instruction"]
        if r.get("input"):
            user_text += f"\n\n{r['input']}"
        out.append(
            {
                "conversations": [
                    {"from": "human", "value": user_text},
                    {"from": "gpt", "value": r["output"]},
                ]
            }
        )
    return out


def train(dataset_path: str, base_model: str, output: str, epochs: int):
    try:
        from unsloth import FastLanguageModel, is_bfloat16_supported
        from unsloth import UnslothTrainer, UnslothTrainingArguments
        from datasets import load_dataset
        import torch
    except ImportError as e:
        print(f"[E6] ERROR: falta dependencia: {e}")
        print("Instala Unsloth primero (ver cabecera del script).")
        sys.exit(1)

    max_seq_length = 2048
    dtype = None
    load_in_4bit = True  # QLoRA: cabe en 8 GB VRAM

    # 1. Cargar modelo base en 4-bit.
    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=base_model,
        max_seq_length=max_seq_length,
        dtype=dtype,
        load_in_4bit=load_in_4bit,
    )

    # 2. Configurar LoRA.
    model = FastLanguageModel.get_peft_model(
        model,
        r=16,
        target_modules=[
            "q_proj",
            "k_proj",
            "v_proj",
            "o_proj",
            "gate_proj",
            "up_proj",
            "down_proj",
        ],
        lora_alpha=32,
        lora_dropout=0,
        bias="none",
        use_gradient_checkpointing="unsloth",
        random_state=3407,
        use_rslora=False,
        loftq_config=None,
    )

    # 3. Cargar y formatear dataset.
    rows = load_dataset(dataset_path)
    chat = build_chat_dataset(rows)
    tmp_json = "/tmp/nexus_lora_dataset.json"
    with open(tmp_json, "w", encoding="utf-8") as f:
        for c in chat:
            f.write(json.dumps(c) + "\n")

    dataset = load_dataset("json", data_files=tmp_json, split="train")

    def format_prompts(examples):
        texts = [
            tokenizer.apply_chat_template(
                convo["conversations"], tokenize=False, add_generation_prompt=False
            )
            for convo in examples
        ]
        return {"text": texts}

    dataset = dataset.map(format_prompts, batched=True, remove_columns=["conversations"])

    # 4. Entrenar.
    trainer = UnslothTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        dataset_text_field="text",
        max_seq_length=max_seq_length,
        args=UnslothTrainingArguments(
            per_device_train_batch_size=1,
            gradient_accumulation_steps=8,  # lote efectivo 8
            warmup_steps=5,
            num_train_epochs=epochs,
            learning_rate=2e-4,
            fp16=not is_bfloat16_supported(),
            bf16=is_bfloat16_supported(),
            logging_steps=1,
            optim="adamw_8bit",
            weight_decay=0.01,
            lr_scheduler_type="cosine",
            seed=3407,
            output_dir=output,
            report_to="none",
        ),
    )

    trainer.train()

    # 5. Guardar LoRA + fused en 4-bit.
    model.save_pretrained_merged(output, tokenizer, save_method="merged_16bit")
    print(f"[E6] Modelo guardado en {output}")
    print(f"[E6] Conviértelo a GGUF: llamacpp/convert_hf_to_gguf.py {output}")


def main():
    parser = argparse.ArgumentParser(
        description="Fine-tuning LoRA de Qwen 2.5 con el corpus de scraping NEXUS"
    )
    parser.add_argument(
        "--dataset",
        required=True,
        help="Path al dataset instruct .jsonl (instruction/input/output)",
    )
    parser.add_argument(
        "--base",
        default="Qwen/Qwen2.5-7B-Instruct",
        help="Modelo base (HF id). Alternativa: Qwen/Qwen2.5-7B-Instruct-abliterated",
    )
    parser.add_argument("--output", default="models/nexus-qwen-lora", help="Dir de salida")
    parser.add_argument("--epochs", type=int, default=3, help="Número de épocas (3-5)")
    args = parser.parse_args()

    train(args.dataset, args.base, args.output, args.epochs)


if __name__ == "__main__":
    main()
