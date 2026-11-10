from __future__ import annotations

import json
import mimetypes
import os
import re
import shutil
import subprocess
import tempfile
import threading
from collections import OrderedDict
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.parse import quote

from flask import Flask, jsonify, render_template, request, send_file
from werkzeug.utils import secure_filename

try:
    from PIL import Image, ImageOps, UnidentifiedImageError
except ImportError:  # The editor can open without Pillow; conversion/validation report the dependency.
    Image = None
    ImageOps = None
    UnidentifiedImageError = Exception

APP_DIR = Path(__file__).resolve().parent
DEFAULT_PROJECT = APP_DIR.parents[1]
PROJECT_ROOT: Path | None = None
PROJECT_LOCK = threading.RLock()

app = Flask(__name__)
app.config["MAX_CONTENT_LENGTH"] = 256 * 1024 * 1024  # 256 MB por upload

IMAGE_EXTENSIONS = {".png", ".jpg", ".jpeg", ".bmp", ".gif", ".webp", ".tif", ".tiff"}
AUDIO_EXTENSIONS = {".mp3", ".wav", ".ogg", ".flac", ".m4a"}


def api_error(message: str, status: int = 400):
    return jsonify({"ok": False, "error": str(message)}), status


def current_root() -> Path:
    with PROJECT_LOCK:
        root = PROJECT_ROOT
    if root is None or not root.is_dir():
        raise RuntimeError("Nenhum projeto aberto. Selecione a pasta raiz do Brazil Fight Combat.")
    return root


def set_project(path_value: str) -> Path:
    global PROJECT_ROOT
    candidate = Path(path_value).expanduser()
    if not candidate.is_absolute():
        candidate = (Path.cwd() / candidate)
    candidate = candidate.resolve()
    if not candidate.is_dir():
        raise ValueError(f"Diretório não encontrado: {candidate}")

    # O projeto é identificado pelos arquivos reais desta distribuição.
    markers = [candidate / "Cargo.toml", candidate / "chars" / "chars.json"]
    if not all(p.is_file() for p in markers):
        missing = [str(p.relative_to(candidate)) for p in markers if not p.is_file()]
        raise ValueError("A pasta selecionada não parece ser a raiz do BFC. Arquivos obrigatórios ausentes: " + ", ".join(missing))
    with PROJECT_LOCK:
        PROJECT_ROOT = candidate
    return candidate


def safe_path(relative: str | Path) -> Path:
    root = current_root()
    raw = str(relative).replace("\\", "/")
    pure = PurePosixPath(raw)
    if pure.is_absolute() or ".." in pure.parts:
        raise ValueError("Caminho relativo inválido.")
    path = (root / Path(*pure.parts)).resolve()
    if path != root and root not in path.parents:
        raise ValueError("Acesso fora da pasta do projeto bloqueado.")
    return path


def relative_posix(path: Path) -> str:
    return path.resolve().relative_to(current_root()).as_posix()


def atomic_write(path: Path, data: bytes, backup: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if backup and path.exists():
        backup_path = path.with_name(path.name + ".bak")
        if not backup_path.exists():
            shutil.copy2(path, backup_path)
    fd, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temp_name, path)
    finally:
        if os.path.exists(temp_name):
            os.unlink(temp_name)


def read_json_file(path: Path, default: Any = None) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8-sig"))
    except FileNotFoundError:
        return default
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        raise ValueError(f"JSON inválido em {relative_posix(path)}: {exc}") from exc


def write_json_file(path: Path, value: Any, backup: bool = True) -> None:
    content = json.dumps(value, ensure_ascii=False, indent=2) + "\n"
    atomic_write(path, content.encode("utf-8"), backup=backup)


def parse_ini(path: Path) -> list[dict[str, Any]]:
    """Parser INI simples que mantém ordem e suporta os arquivos .ini do motor."""
    if not path.exists():
        return []
    sections: list[dict[str, Any]] = []
    current: dict[str, Any] = {"name": "", "values": OrderedDict()}
    sections.append(current)
    for raw in path.read_text(encoding="utf-8-sig", errors="replace").splitlines():
        line = raw.strip()
        if not line or line.startswith(("#", ";")):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = {"name": line[1:-1].strip(), "values": OrderedDict()}
            sections.append(current)
            continue
        if "=" in line:
            key, value = line.split("=", 1)
            current["values"][key.strip()] = value.strip()
    return [s for s in sections if s["name"] or s["values"]]


def section_map(path: Path) -> OrderedDict[str, OrderedDict[str, str]]:
    result: OrderedDict[str, OrderedDict[str, str]] = OrderedDict()
    for section in parse_ini(path):
        result[section["name"]] = section["values"]
    return result


def save_stages(stages: list[dict[str, Any]]) -> None:
    root = current_root()
    lines = ["# Brazil Fight Combat - cenarios editados pelo BFC Game Studio", ""]
    for index, stage in enumerate(stages):
        stage_id = str(stage.get("id", "")).strip()
        if not re.fullmatch(r"[a-zA-Z0-9_-]{1,48}", stage_id):
            raise ValueError(f"ID de fase inválido: {stage_id!r}")
        name = str(stage.get("name", stage_id)).replace("\n", " ").strip()
        image = str(stage.get("image", "")).replace("\\", "/").strip()
        if image:
            safe_path(image)
        music = str(stage.get("music") or "null").strip()
        if music.lower() in {"none", "null", ""}:
            music = "null"
        ground = float(stage.get("ground", 470))
        if not 0 <= ground <= 540:
            raise ValueError(f"Ground da fase {stage_id} deve estar entre 0 e 540.")
        lines.extend([
            f"[stage.{index:02d}]",
            f"id={stage_id}",
            f"name={name}",
            f"image={image}",
            f"music={music}",
            f"ground={ground:g}",
            "",
        ])
    atomic_write(root / "data" / "stages.ini", ("\n".join(lines).rstrip() + "\n").encode("utf-8"), backup=True)


def load_stages() -> list[dict[str, Any]]:
    root = current_root()
    data = section_map(root / "data" / "stages.ini")
    stages = []
    for section, values in data.items():
        if not section.startswith("stage."):
            continue
        stage_id = values.get("id", "").strip()
        if not stage_id:
            continue
        image = values.get("image", "")
        image_exists = bool(image and safe_path(image).is_file())
        stages.append({
            "id": stage_id,
            "name": values.get("name", stage_id),
            "image": image,
            "image_url": f"/api/file/{quote(image, safe='/')}" if image_exists else "",
            "image_exists": image_exists,
            "music": values.get("music", "null"),
            "ground": float(values.get("ground", 470) or 470),
        })
    return stages


def load_moves() -> dict[str, dict[str, Any]]:
    root = current_root()
    raw = section_map(root / "data" / "moves.ini")
    defaults = {
        "punch": {"startup": 5, "active": 4, "recovery": 9, "damage": 6, "reach": 75.0, "y_offset": 170.0, "stun": 16, "knockback": 3.0},
        "kick": {"startup": 8, "active": 5, "recovery": 13, "damage": 9, "reach": 95.0, "y_offset": 110.0, "stun": 20, "knockback": 5.0},
        "sweep": {"startup": 7, "active": 5, "recovery": 14, "damage": 8, "reach": 100.0, "y_offset": 35.0, "stun": 30, "knockback": 4.0},
        "special": {"startup": 14, "active": 7, "recovery": 22, "damage": 16, "reach": 140.0, "y_offset": 160.0, "stun": 28, "knockback": 9.0},
    }
    result = {}
    for move_id, default_values in defaults.items():
        values = raw.get(move_id, {})
        current = {}
        for key, default in default_values.items():
            try:
                current[key] = float(values.get(key, default)) if key in {"reach", "y_offset", "knockback"} else int(float(values.get(key, default)))
            except (TypeError, ValueError):
                current[key] = default
        result[move_id] = current
    return result


def save_moves(moves: dict[str, dict[str, Any]]) -> None:
    expected = {"punch", "kick", "sweep", "special"}
    if not isinstance(moves, dict):
        raise ValueError("Formato de golpes inválido.")
    lines = ["# Brazil Fight Combat - parametros globais de golpes", ""]
    fields = ("startup", "active", "recovery", "damage", "reach", "y_offset", "stun", "knockback")
    for move_id in ("punch", "kick", "sweep", "special"):
        values = moves.get(move_id, {})
        lines.append(f"[{move_id}]")
        for key in fields:
            try:
                value = float(values.get(key, 0))
            except (TypeError, ValueError):
                raise ValueError(f"Valor inválido em {move_id}.{key}")
            if key in {"startup", "active", "recovery", "damage", "stun"}:
                value = int(value)
            lines.append(f"{key}={value:g}")
        lines.append("")
    path = current_root() / "data" / "moves.ini"
    atomic_write(path, ("\n".join(lines).rstrip() + "\n").encode("utf-8"), backup=True)


def load_engine() -> dict[str, Any]:
    root = current_root()
    flat: dict[str, str] = {}
    for section in parse_ini(root / "data" / "engine.ini"):
        flat.update(section["values"])
    defaults: dict[str, Any] = {
        "ground": 470, "round_seconds": 60, "rounds_to_win": 2, "cpu_difficulty": 1,
        "gravity": 1.0, "jump_velocity": 15.0, "stage_name": "Brazil Fight Combat",
        "sky_red": 25, "sky_green": 34, "sky_blue": 67,
        "floor_red": 52, "floor_green": 58, "floor_blue": 68,
    }
    result: dict[str, Any] = {}
    for key, default in defaults.items():
        value = flat.get(key, default)
        try:
            result[key] = type(default)(value)
        except (TypeError, ValueError):
            result[key] = default
    result["width"] = 960
    result["height"] = 540
    return result


def save_engine(values: dict[str, Any]) -> None:
    root = current_root()
    path = root / "data" / "engine.ini"
    defaults = load_engine()
    fields = ["ground", "round_seconds", "rounds_to_win", "cpu_difficulty", "gravity", "jump_velocity", "stage_name", "sky_red", "sky_green", "sky_blue", "floor_red", "floor_green", "floor_blue"]
    current: OrderedDict[str, str] = OrderedDict()
    for section in parse_ini(path):
        current.update(section["values"])
    for key in fields:
        if key not in values:
            continue
        raw = values[key]
        if key == "stage_name":
            current[key] = str(raw).replace("\n", " ").strip()
        elif key in {"gravity", "jump_velocity"}:
            value = float(raw)
            if not 0.01 <= value <= 100:
                raise ValueError(f"{key} fora do intervalo permitido.")
            current[key] = f"{value:g}"
        elif key in {"ground"}:
            value = float(raw)
            if not 0 <= value <= 540:
                raise ValueError("ground deve estar entre 0 e 540.")
            current[key] = f"{value:g}"
        else:
            value = int(float(raw))
            limits = {"round_seconds": (1, 999), "rounds_to_win": (1, 5), "cpu_difficulty": (0, 2)}
            if key.startswith(("sky_", "floor_")):
                limits[key] = (0, 255)
            low, high = limits.get(key, (0, 9999))
            if not low <= value <= high:
                raise ValueError(f"{key} deve estar entre {low} e {high}.")
            current[key] = str(value)
    lines = ["# Regras de simulacao do Brazil Fight Combat. Resolucao logica fixa: 960x540."]
    lines.extend(f"{key}={value}" for key, value in current.items())
    atomic_write(path, ("\n".join(lines) + "\n").encode("utf-8"), backup=True)


def get_roster() -> list[str]:
    root = current_root()
    data = read_json_file(root / "chars" / "chars.json", {"personagens": []}) or {"personagens": []}
    roster = data.get("personagens", []) if isinstance(data, dict) else []
    return [str(item) for item in roster if isinstance(item, str) and re.fullmatch(r"[a-zA-Z0-9_-]{1,48}", item)]


def character_path(char_id: str) -> Path:
    if not re.fullmatch(r"[a-zA-Z0-9_-]{1,48}", char_id):
        raise ValueError("ID de personagem inválido.")
    return safe_path(f"chars/{char_id}/character.json")


def is_image_file(path: Path) -> bool:
    if not path.is_file() or path.name.lower().endswith((".bak", ".deleted")) or path.name.startswith(".bfc_"):
        return False
    if Image is None:
        return path.suffix.lower() in IMAGE_EXTENSIONS
    try:
        with Image.open(path) as probe:
            probe.verify()
        return True
    except Exception:
        # Extensoes de imagem conhecidas continuam visiveis para que o validador
        # possa apontar o arquivo corrompido na interface.
        return path.suffix.lower() in IMAGE_EXTENSIONS


def image_files(folder: Path) -> list[Path]:
    if not folder.is_dir():
        return []
    return sorted([p for p in folder.iterdir() if p.is_file() and is_image_file(p)], key=lambda p: p.name.lower())


def api_image_url(path: Path) -> str:
    try:
        return "/api/file/" + quote(relative_posix(path), safe="/")
    except Exception:
        return ""


def list_characters() -> list[dict[str, Any]]:
    root = current_root()
    items = []
    for char_id in get_roster():
        folder = safe_path(f"chars/{char_id}")
        cfg_path = character_path(char_id)
        try:
            cfg = read_json_file(cfg_path, {}) or {}
        except ValueError as exc:
            cfg = {"name": char_id, "_error": str(exc)}
        art = cfg.get("art", {}) if isinstance(cfg, dict) else {}
        face_path = folder / str(art.get("face", "face.png"))
        sprite_path = folder / str(art.get("sprite", "sprite.png"))
        image_count = len(image_files(folder))
        items.append({
            "id": char_id,
            "name": cfg.get("name", char_id),
            "hp": (cfg.get("stats") or {}).get("hp", 200),
            "speed": (cfg.get("stats") or {}).get("speed", 4.5),
            "power": (cfg.get("stats") or {}).get("power", 1.0),
            "face_url": api_image_url(face_path) if face_path.is_file() else api_image_url(sprite_path) if sprite_path.is_file() else "",
            "sprite_url": api_image_url(sprite_path) if sprite_path.is_file() else "",
            "image_count": image_count,
            "error": cfg.get("_error", ""),
        })
    return items


def character_detail(char_id: str) -> dict[str, Any]:
    root = current_root()
    folder = safe_path(f"chars/{char_id}")
    cfg = read_json_file(character_path(char_id), {})
    if not isinstance(cfg, dict):
        raise ValueError(f"character.json inválido para {char_id}.")
    files = image_files(folder)
    frames = [{"name": p.name, "url": api_image_url(p), "size": p.stat().st_size} for p in files]
    return {"id": char_id, "config": cfg, "frames": frames, "directory": relative_posix(folder)}


def atomic_update_text_references(replacements: dict[str, str]) -> list[str]:
    root = current_root()
    changed: list[str] = []
    if not replacements:
        return changed
    # Caminhos completos são atualizados globalmente; nomes locais somente
    # no character.json pertencente à mesma pasta do sprite convertido.
    candidates = list((root / "chars").rglob("*.json")) + [root / "data" / "stages.ini"]
    for path in candidates:
        if not path.is_file():
            continue
        try:
            original = path.read_text(encoding="utf-8-sig")
        except (OSError, UnicodeDecodeError):
            continue
        updated = original
        for old, new in replacements.items():
            updated = updated.replace(old, new)
            old_path = PurePosixPath(old)
            new_path = PurePosixPath(new)
            try:
                config_parent = path.parent.resolve().relative_to(root.resolve()).as_posix()
            except ValueError:
                config_parent = ""
            if path.name == "character.json" and config_parent == old_path.parent.as_posix():
                updated = re.sub(rf'(?<=["\'=]){re.escape(old_path.name)}(?=["\'])', new_path.name, updated)
        if updated != original:
            atomic_write(path, updated.encode("utf-8"), backup=True)
            changed.append(relative_posix(path))
    return changed


def convert_project_images() -> dict[str, Any]:
    root = current_root()
    if Image is None:
        raise RuntimeError("Pillow não está instalado. Execute INICIAR_BFC_STUDIO.bat para instalar as dependências.")

    scan_roots: list[tuple[str, Path]] = []
    chars_root = root / "chars"
    if chars_root.is_dir():
        # Converte todos os diretórios em chars/, inclusive personagens ainda
        # não registrados no roster, para não deixar sprites de fora.
        for folder in sorted((p for p in chars_root.iterdir() if p.is_dir() and not p.name.startswith(".")), key=lambda p: p.name.lower()):
            scan_roots.append((f"personagem:{folder.name}", folder))
    stages_root = root / "assets" / "images" / "stages"
    if stages_root.is_dir():
        scan_roots.append(("stage", stages_root))

    converted: list[dict[str, str]] = []
    skipped: list[str] = []
    errors: list[str] = []
    collisions: list[str] = []
    replacements: dict[str, str] = {}

    for kind, folder in scan_roots:
        for source in sorted(folder.rglob("*")):
            if not source.is_file() or source.name.lower().endswith((".bak", ".deleted")) or source.name.startswith(".bfc_"):
                continue
            try:
                with Image.open(source) as im:
                    detected = (im.format or "UNKNOWN").upper()
                    if detected == "PNG" and source.suffix.lower() == ".png":
                        skipped.append(relative_posix(source))
                        continue
                    im.seek(0)  # GIF/TIFF animado: converte o primeiro quadro.
                    frame = ImageOps.exif_transpose(im.copy())
                    if frame.mode not in ("1", "L", "LA", "P", "RGB", "RGBA", "I", "I;16"):
                        frame = frame.convert("RGBA" if "A" in frame.getbands() else "RGB")
                    target = source if source.suffix.lower() == ".png" else source.with_suffix(".png")
                    if target.exists() and target.resolve() != source.resolve():
                        collisions.append(f"{relative_posix(source)} -> {relative_posix(target)} (destino existente)")
                        continue
                    fd, temp_name = tempfile.mkstemp(prefix=".bfc_png_", suffix=".png", dir=source.parent)
                    os.close(fd)
                    try:
                        frame.save(temp_name, format="PNG", optimize=True)
                        with open(temp_name, "rb") as handle:
                            os.fsync(handle.fileno())
                        if target.resolve() == source.resolve():
                            backup = source.with_name(source.name + ".original.bak")
                            if not backup.exists():
                                shutil.copy2(source, backup)
                        old_rel = relative_posix(source)
                        new_rel = relative_posix(target)
                        os.replace(temp_name, target)
                        converted.append({"from": old_rel, "to": new_rel, "format": detected, "group": kind})
                        if old_rel != new_rel:
                            replacements[old_rel] = new_rel
                            # JSON art fields are often relative to chars/<id>/.
                            if kind.startswith("personagem:"):
                                replacements[PurePosixPath(old_rel).name] = PurePosixPath(new_rel).name
                    finally:
                        if os.path.exists(temp_name):
                            os.unlink(temp_name)
            except (UnidentifiedImageError, OSError, ValueError) as exc:
                # JSON, TXT, atlas e outros metadados não são imagens; são ignorados.
                if source.suffix.lower() in IMAGE_EXTENSIONS:
                    errors.append(f"{relative_posix(source)}: {exc}")

    updated_files = atomic_update_text_references(replacements)
    return {
        "converted": converted,
        "skipped": skipped,
        "collisions": collisions,
        "errors": errors,
        "references_updated": updated_files,
        "summary": {"converted": len(converted), "already_png": len(skipped), "collisions": len(collisions), "errors": len(errors)},
        "note": "O conversor não aplica chroma key. A transparência/fundo dos sprites é controlada pelo motor e pela configuração chroma de cada personagem. Cenas nunca recebem remoção de cor.",
    }


def scan_audio() -> dict[str, list[dict[str, str]]]:
    root = current_root()
    groups = {
        "music": root / "assets" / "audio" / "music",
        "sfx": root / "assets" / "audio" / "sfx",
        "voice": root / "assets" / "audio" / "voice",
    }
    result: dict[str, list[dict[str, str]]] = {}
    for key, folder in groups.items():
        files = []
        if folder.is_dir():
            for p in sorted(folder.rglob("*"), key=lambda x: str(x).lower()):
                if p.is_file() and p.suffix.lower() in AUDIO_EXTENSIONS:
                    rel = relative_posix(p)
                    files.append({"name": p.name, "path": rel, "url": "/api/file/" + quote(rel, safe="/"), "group": p.parent.name})
        result[key] = files
    return result


def count_files(folder: Path, suffixes: set[str] | None = None) -> int:
    if not folder.is_dir():
        return 0
    if suffixes == IMAGE_EXTENSIONS:
        return sum(1 for p in folder.rglob("*") if p.is_file() and is_image_file(p))
    return sum(1 for p in folder.rglob("*") if p.is_file() and (suffixes is None or p.suffix.lower() in suffixes))


def validate_project() -> dict[str, Any]:
    root = current_root()
    issues: list[dict[str, str]] = []
    checks = ["Cargo.toml", "chars/chars.json", "data/engine.ini", "data/stages.ini", "data/moves.ini"]
    for item in checks:
        if not (root / item).is_file():
            issues.append({"level": "error", "path": item, "message": "Arquivo necessário pelo motor não encontrado."})
    try:
        roster = get_roster()
    except Exception as exc:
        roster = []
        issues.append({"level": "error", "path": "chars/chars.json", "message": str(exc)})
    for char_id in roster:
        path = root / "chars" / char_id / "character.json"
        if not path.is_file():
            issues.append({"level": "error", "path": relative_posix(path), "message": "Personagem registrado sem character.json."})
            continue
        try:
            cfg = read_json_file(path, {})
            for key in ("art",):
                if not isinstance(cfg.get(key, {}), dict):
                    issues.append({"level": "warning", "path": relative_posix(path), "message": f"Campo {key} não é um objeto."})
            for art_key in ("sprite", "face"):
                value = (cfg.get("art") or {}).get(art_key)
                if value and not (path.parent / value).is_file():
                    issues.append({"level": "warning", "path": relative_posix(path), "message": f"Arte {art_key} não encontrada: {value}"})
        except Exception as exc:
            issues.append({"level": "error", "path": relative_posix(path), "message": str(exc)})
    try:
        stages = load_stages()
        for stage in stages:
            if not stage["image_exists"]:
                issues.append({"level": "warning", "path": "data/stages.ini", "message": f"Imagem não encontrada na fase {stage['id']}: {stage['image']}"})
    except Exception as exc:
        issues.append({"level": "error", "path": "data/stages.ini", "message": str(exc)})

    # Relatório de imagens: tenta verificar decodificação, sem modificar nada.
    if Image is not None:
        for folder in [root / "assets" / "images" / "stages"] + [root / "chars" / c for c in roster]:
            for path in image_files(folder):
                try:
                    with Image.open(path) as im:
                        im.verify()
                except Exception as exc:
                    issues.append({"level": "error", "path": relative_posix(path), "message": f"Imagem inválida ou corrompida: {exc}"})
    return {"issues": issues, "summary": {"errors": sum(x["level"] == "error" for x in issues), "warnings": sum(x["level"] == "warning" for x in issues), "characters": len(roster), "stages": len(load_stages())}}


@app.get("/")
def index():
    return render_template("index.html")


@app.get("/api/status")
def status():
    with PROJECT_LOCK:
        root = PROJECT_ROOT
    if root is None:
        # Automatically attach to the project when installed inside tools/BFC_Game_Studio.
        try:
            root = set_project(str(DEFAULT_PROJECT))
        except Exception:
            root = None
    return jsonify({"ok": True, "project": str(root) if root else "", "opened": bool(root), "default_project": str(DEFAULT_PROJECT)})


@app.post("/api/project/open")
def open_project():
    try:
        data = request.get_json(force=True) or {}
        root = set_project(str(data.get("path", "")))
        return jsonify({"ok": True, "project": str(root), "name": root.name})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/dashboard")
def dashboard():
    try:
        root = current_root()
        chars = get_roster()
        stages = load_stages()
        audio = scan_audio()
        return jsonify({
            "project": str(root),
            "characters": len(chars),
            "stages": len(stages),
            "sprites": sum(count_files(folder, IMAGE_EXTENSIONS) for folder in (root / "chars").iterdir() if folder.is_dir() and not folder.name.startswith(".")),
            "stage_images": count_files(root / "assets" / "images" / "stages", IMAGE_EXTENSIONS),
            "music": len(audio["music"]),
            "sfx": len(audio["sfx"]),
            "voices": len(audio["voice"]),
        })
    except Exception as exc:
        return api_error(exc, 400)


@app.get("/api/characters")
def characters_get():
    try:
        return jsonify({"items": list_characters()})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/characters/<char_id>")
def character_get(char_id: str):
    try:
        return jsonify(character_detail(char_id))
    except Exception as exc:
        return api_error(exc)


@app.put("/api/characters/<char_id>")
def character_save(char_id: str):
    try:
        data = request.get_json(force=True)
        if not isinstance(data, dict):
            raise ValueError("O corpo da requisição deve ser um objeto JSON.")
        path = character_path(char_id)
        # Preserva campos desconhecidos enviados pelo editor, mas não permite ID/caminho externo.
        if "name" not in data or not str(data["name"]).strip():
            raise ValueError("O personagem precisa ter um nome.")
        if data.get("stage") == "":
            data["stage"] = None
        if data.get("music") == "":
            data["music"] = None
        stats = data.setdefault("stats", {})
        stats["speed"] = max(1.0, min(12.0, float(stats.get("speed", 4.5))))
        stats["power"] = max(0.1, min(5.0, float(stats.get("power", 1.0))))
        stats["hp"] = max(1, min(5000, int(float(stats.get("hp", 200)))))
        for section in ("art", "animations", "effects", "moves", "stats", "anchor", "chroma"):
            if section in data and not isinstance(data[section], dict):
                raise ValueError(f"O campo {section} deve ser um objeto JSON.")
        write_json_file(path, data, backup=True)
        return jsonify({"ok": True, "character": character_detail(char_id)})
    except Exception as exc:
        return api_error(exc)


@app.post("/api/characters")
def character_create():
    try:
        root = current_root()
        data = request.get_json(force=True) or {}
        char_id = str(data.get("id", "")).strip().lower().replace(" ", "-")
        name = str(data.get("name", "Novo Personagem")).strip()
        if not re.fullmatch(r"[a-z0-9_-]{2,48}", char_id):
            raise ValueError("Use um ID com letras minúsculas, números, hífen ou sublinhado.")
        if char_id in get_roster() or (root / "chars" / char_id).exists():
            raise ValueError("Já existe um personagem com esse ID.")
        folder = root / "chars" / char_id
        folder.mkdir(parents=True, exist_ok=False)
        template = {
            "name": name,
            "stats": {"speed": 4.5, "power": 1.0, "hp": 200},
            "stage": None, "music": None, "voice": char_id, "voices": {},
            "art": {"sprite": "sprite.png", "face": "face.png"},
            "size": 300, "anchor": {"x": 0.5, "y": 0.92}, "facing": "right",
            "chroma": {"color": "#00ff00", "tolerance": 90, "softness": 60, "maxHeight": 560},
            "animations": {"idle": {"prefix": "idle", "fps": 8, "mode": "loop"}, "attack": {"prefix": "attack", "fps": 12, "mode": "progress"}, "kick": {"prefix": "kick", "fps": 12, "mode": "progress"}, "jump": {"prefix": "jump", "mode": "vy"}, "walk": {"prefix": "walk", "fps": 12, "mode": "loop"}, "run": {"prefix": "run", "fps": 16, "mode": "loop"}},
            "effects": {}, "moves": {}, "rig": {"neck": 0.24, "hip": 0.57, "split": 0.5, "flip": 0, "skirt": 0},
        }
        write_json_file(folder / "character.json", template, backup=False)
        roster_path = root / "chars" / "chars.json"
        roster = read_json_file(roster_path, {"personagens": []})
        roster.setdefault("personagens", []).append(char_id)
        write_json_file(roster_path, roster, backup=True)
        return jsonify({"ok": True, "id": char_id, "character": character_detail(char_id)}), 201
    except Exception as exc:
        return api_error(exc)


@app.delete("/api/characters/<char_id>")
def character_delete(char_id: str):
    try:
        root = current_root()
        roster_path = root / "chars" / "chars.json"
        roster = read_json_file(roster_path, {"personagens": []})
        if char_id not in roster.get("personagens", []):
            raise ValueError("Personagem não está registrado no elenco.")
        roster["personagens"].remove(char_id)
        write_json_file(roster_path, roster, backup=True)
        # A pasta é preservada: remover do elenco não apaga sprites/histórias.
        return jsonify({"ok": True, "message": "Personagem removido do elenco. A pasta e os recursos foram preservados."})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/stages")
def stages_get():
    try:
        return jsonify({"items": load_stages()})
    except Exception as exc:
        return api_error(exc)


@app.put("/api/stages")
def stages_save():
    try:
        data = request.get_json(force=True) or {}
        stages = data.get("items", [])
        if not isinstance(stages, list):
            raise ValueError("A lista de fases é inválida.")
        save_stages(stages)
        return jsonify({"ok": True, "items": load_stages()})
    except Exception as exc:
        return api_error(exc)


@app.post("/api/stages")
def stage_create():
    try:
        stages = load_stages()
        data = request.get_json(force=True) or {}
        stage_id = str(data.get("id", "nova-fase")).strip().lower()
        if any(stage["id"] == stage_id for stage in stages):
            raise ValueError("Já existe uma fase com esse ID.")
        stages.append({"id": stage_id, "name": data.get("name", "Nova Fase"), "image": "", "music": "null", "ground": 470})
        save_stages(stages)
        return jsonify({"ok": True, "items": load_stages()}), 201
    except Exception as exc:
        return api_error(exc)


@app.get("/api/moves")
def moves_get():
    try:
        return jsonify({"moves": load_moves()})
    except Exception as exc:
        return api_error(exc)


@app.put("/api/moves")
def moves_put():
    try:
        save_moves((request.get_json(force=True) or {}).get("moves", {}))
        return jsonify({"ok": True, "moves": load_moves()})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/rules")
def rules_get():
    try:
        return jsonify({"rules": load_engine()})
    except Exception as exc:
        return api_error(exc)


@app.put("/api/rules")
def rules_put():
    try:
        save_engine((request.get_json(force=True) or {}).get("rules", {}))
        return jsonify({"ok": True, "rules": load_engine()})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/audio")
def audio_get():
    try:
        return jsonify(scan_audio())
    except Exception as exc:
        return api_error(exc)


@app.get("/api/stories")
def stories_get():
    try:
        root = current_root()
        items = []
        folder = root / "story"
        if folder.is_dir():
            for story_dir in sorted([p for p in folder.iterdir() if p.is_dir()], key=lambda p: p.name.lower()):
                story_path = story_dir / "historia.txt"
                items.append({"id": story_dir.name, "exists": story_path.is_file(), "text": story_path.read_text(encoding="utf-8", errors="replace") if story_path.is_file() else ""})
        return jsonify({"items": items})
    except Exception as exc:
        return api_error(exc)


@app.put("/api/stories/<char_id>")
def story_put(char_id: str):
    try:
        if not re.fullmatch(r"[a-zA-Z0-9_-]{1,48}", char_id):
            raise ValueError("ID inválido.")
        text = str((request.get_json(force=True) or {}).get("text", ""))
        if len(text) > 200_000:
            raise ValueError("O texto da história excede 200 KB.")
        path = safe_path(f"story/{char_id}/historia.txt")
        atomic_write(path, text.encode("utf-8"), backup=True)
        return jsonify({"ok": True})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/assets")
def assets_get():
    try:
        root = current_root()
        category = request.args.get("category", "images")
        categories = {
            "stages": "assets/images/stages", "effects": "assets/images/effects",
            "music": "assets/audio/music", "sfx": "assets/audio/sfx", "voice": "assets/audio/voice",
        }
        rel = categories.get(category)
        if not rel:
            raise ValueError("Categoria de recursos inválida.")
        folder = safe_path(rel)
        items = []
        if folder.is_dir():
            for path in sorted(folder.rglob("*"), key=lambda p: str(p).lower()):
                if path.is_file():
                    suffix = path.suffix.lower()
                    if category in {"stages", "effects"} and not is_image_file(path):
                        continue
                    if category in {"music", "sfx", "voice"} and suffix not in AUDIO_EXTENSIONS:
                        continue
                    rp = relative_posix(path)
                    items.append({"name": path.name, "path": rp, "url": "/api/file/" + quote(rp, safe="/"), "bytes": path.stat().st_size})
        return jsonify({"category": category, "items": items})
    except Exception as exc:
        return api_error(exc)


@app.post("/api/upload")
def upload_asset():
    try:
        current_root()
        category = request.form.get("category", "")
        allowed = {
            "stages": ("assets/images/stages", IMAGE_EXTENSIONS),
            "effects": ("assets/images/effects", IMAGE_EXTENSIONS),
            "music": ("assets/audio/music", AUDIO_EXTENSIONS),
            "sfx": ("assets/audio/sfx", AUDIO_EXTENSIONS),
            "voice": ("assets/audio/voice", AUDIO_EXTENSIONS),
        }
        if category.startswith("character:"):
            char_id = category.split(":", 1)[1]
            allowed[category] = (f"chars/{char_id}", IMAGE_EXTENSIONS | AUDIO_EXTENSIONS | {".json", ".txt"})
        if category.startswith("voice:"):
            char_id = category.split(":", 1)[1]
            allowed[category] = (f"assets/audio/voice/{char_id}", AUDIO_EXTENSIONS)
        if category not in allowed:
            raise ValueError("Destino de upload inválido.")
        if "file" not in request.files:
            raise ValueError("Nenhum arquivo enviado.")
        upload = request.files["file"]
        name = secure_filename(upload.filename or "")
        if not name:
            raise ValueError("Nome de arquivo inválido.")
        folder_rel, extensions = allowed[category]
        if Path(name).suffix.lower() not in extensions:
            is_detected_image = False
            if category in {"stages", "effects"} or category.startswith("character:"):
                if Image is not None:
                    try:
                        upload.stream.seek(0)
                        with Image.open(upload.stream) as probe:
                            probe.verify()
                        upload.stream.seek(0)
                        is_detected_image = True
                    except Exception:
                        upload.stream.seek(0)
            if not is_detected_image:
                raise ValueError(f"Extensão não permitida ou formato não reconhecido: {Path(name).suffix}")
        folder = safe_path(folder_rel)
        folder.mkdir(parents=True, exist_ok=True)
        target = folder / name
        stem, suffix = target.stem, target.suffix
        index = 1
        while target.exists():
            target = folder / f"{stem}_{index}{suffix}"
            index += 1
        upload.save(target)
        rel = relative_posix(target)
        return jsonify({"ok": True, "path": rel, "url": "/api/file/" + quote(rel, safe="/")})
    except Exception as exc:
        return api_error(exc)


@app.delete("/api/file")
def delete_file():
    try:
        data = request.get_json(force=True) or {}
        path = safe_path(str(data.get("path", "")))
        if not path.is_file():
            raise ValueError("Arquivo não encontrado.")
        root = current_root()
        trash = root / ".bfc_studio_trash"
        trash.mkdir(exist_ok=True)
        target = trash / (path.name + ".deleted")
        i = 1
        while target.exists():
            target = trash / (path.name + f"_{i}.deleted")
            i += 1
        shutil.move(str(path), str(target))
        return jsonify({"ok": True, "message": f"Arquivo movido para .bfc_studio_trash/{target.name}"})
    except Exception as exc:
        return api_error(exc)


@app.get("/api/file/<path:relative>")
def serve_project_file(relative: str):
    try:
        path = safe_path(relative)
        rel = PurePosixPath(relative.replace("\\", "/"))
        if not rel.parts or rel.parts[0] not in {"assets", "chars", "story"}:
            return api_error("Este tipo de arquivo não é servido pelo editor.", 403)
        if not path.is_file():
            return api_error("Arquivo não encontrado.", 404)
        mimetype = mimetypes.guess_type(path.name)[0]
        if Image is not None:
            try:
                with Image.open(path) as probe:
                    mimetype = Image.MIME.get(probe.format, mimetype)
            except Exception:
                pass
        return send_file(path, mimetype=mimetype, conditional=True)
    except Exception as exc:
        return api_error(exc, 404)


@app.get("/api/validation")
def validation_get():
    try:
        return jsonify(validate_project())
    except Exception as exc:
        return api_error(exc)


@app.post("/api/tools/convert-images")
def convert_images():
    try:
        return jsonify({"ok": True, **convert_project_images()})
    except Exception as exc:
        return api_error(exc)


@app.post("/api/game/build")
def build_game():
    try:
        root = current_root()
        result = subprocess.run(["cargo", "build", "--release"], cwd=str(root), capture_output=True, text=True, timeout=900)
        return jsonify({
            "ok": True,
            "build_ok": result.returncode == 0,
            "returncode": result.returncode,
            "stdout": result.stdout[-30000:],
            "stderr": result.stderr[-30000:],
        })
    except FileNotFoundError:
        return api_error("Cargo/Rust não foi encontrado no PATH. Instale Rust e reinicie o BFC Game Studio.")
    except subprocess.TimeoutExpired:
        return api_error("A compilação excedeu o limite de 15 minutos.", 408)
    except Exception as exc:
        return api_error(exc)


@app.post("/api/game/launch")
def launch_game():
    try:
        root = current_root()
        candidates = [root / "target" / "release" / "BrazilFightCombat.exe", root / "BrazilFightCombat.exe", root / "target" / "debug" / "BrazilFightCombat.exe"]
        executable = next((p for p in candidates if p.is_file()), None)
        if executable is None:
            raise ValueError("Executável não encontrado. Compile o jogo com cargo build --release primeiro.")
        if os.name != "nt":
            raise ValueError("A execução integrada está configurada para o executável Windows do BFC.")
        subprocess.Popen([str(executable)], cwd=str(root), close_fds=True)
        return jsonify({"ok": True, "message": f"Jogo iniciado: {executable.name}"})
    except Exception as exc:
        return api_error(exc)


@app.errorhandler(413)
def too_large(_error):
    return api_error("Arquivo excede o limite de upload de 256 MB.", 413)


if __name__ == "__main__":
    try:
        set_project(str(DEFAULT_PROJECT))
    except Exception:
        pass
    print("=" * 64)
    print("BFC GAME STUDIO - Editor Web local")
    print(f"Projeto detectado: {PROJECT_ROOT or 'selecione a pasta no navegador'}")
    print("Acesse: http://127.0.0.1:8765")
    print("Use Ctrl+C para encerrar o servidor.")
    print("=" * 64)
    app.run(host="127.0.0.1", port=8765, debug=False, threaded=True)
