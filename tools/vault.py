"""Descarga, trocea y reensambla los recursos pesados listados en manifest.json.

  python tools/vault.py fetch [--all] [ids...]   # descarga a data/ (in_repo por defecto)
  python tools/vault.py pack [ids...]            # data/ -> vault/ en trozos <95 MB + sha256
  python tools/vault.py unpack [ids...]          # vault/ -> data/ verificando sha256
  python tools/vault.py status
"""
import hashlib, json, shutil, sys, urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA, VAULT = ROOT / "data", ROOT / "vault"
CHUNK = 95 * 1024 * 1024
ITEMS = {i["id"]: i for i in json.loads((ROOT / "manifest.json").read_text())["items"]}


def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 22), b""):
            h.update(b)
    return h.hexdigest()


def download(url, dst):
    dst.parent.mkdir(parents=True, exist_ok=True)
    tmp = dst.with_suffix(dst.suffix + ".part")
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (consciousness-vault)"})
    with urllib.request.urlopen(req) as r, open(tmp, "wb") as f:
        shutil.copyfileobj(r, f, 1 << 22)
    tmp.rename(dst)


def fetch(it):
    out = DATA / it["id"]
    if it["kind"] == "hf_repo":
        from huggingface_hub import snapshot_download
        snapshot_download(it["repo"], local_dir=out)
    elif it["kind"] == "hf_file":
        from huggingface_hub import hf_hub_download
        hf_hub_download(it["repo"], it["file"], local_dir=out)
    else:
        name = it.get("name") or it["url"].rsplit("/", 1)[-1]
        url = it.get("url") or f"https://dataverse.harvard.edu/api/access/datafile/{it['file_id']}"
        if not (out / name).exists():
            download(url, out / name)
    shutil.rmtree(out / ".cache", ignore_errors=True)


def files_of(it):
    base = DATA / it["id"]
    return sorted(p for p in base.rglob("*") if p.is_file())


def pack(it):
    base, vdir = DATA / it["id"], VAULT / it["id"]
    index = []
    for p in files_of(it):
        rel = p.relative_to(base).as_posix()
        dst = vdir / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        size, parts = p.stat().st_size, 0
        if size <= CHUNK:
            shutil.copyfile(p, dst)
        else:
            with open(p, "rb") as f:
                while b := f.read(CHUNK):
                    (dst.parent / f"{dst.name}.part{parts:03d}").write_bytes(b)
                    parts += 1
        index.append({"path": rel, "size": size, "sha256": sha256(p), "parts": parts})
    (vdir / "_index.json").write_text(json.dumps(index, indent=1))


def unpack(it):
    vdir, base = VAULT / it["id"], DATA / it["id"]
    for e in json.loads((vdir / "_index.json").read_text()):
        dst, src = base / e["path"], vdir / e["path"]
        if dst.exists() and dst.stat().st_size == e["size"]:
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        if e["parts"] == 0:
            shutil.copyfile(src, dst)
        else:
            with open(dst, "wb") as f:
                for i in range(e["parts"]):
                    f.write((src.parent / f"{src.name}.part{i:03d}").read_bytes())
        assert sha256(dst) == e["sha256"], f"checksum falla: {dst}"


def main():
    cmd, args = sys.argv[1], [a for a in sys.argv[2:] if not a.startswith("--")]
    everything = "--all" in sys.argv
    ids = args or [k for k, v in ITEMS.items() if v["in_repo"] or everything]
    if cmd == "status":
        for k, v in ITEMS.items():
            print(f"{k:40s} repo={v['in_repo']!s:5s} data={(DATA / k).exists()!s:5s} vault={(VAULT / k / '_index.json').exists()}")
        return
    fn = {"fetch": fetch, "pack": pack, "unpack": unpack}[cmd]
    for k in ids:
        print(cmd, k, flush=True)
        fn(ITEMS[k])


if __name__ == "__main__":
    main()
