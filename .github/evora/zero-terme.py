#!/usr/bin/env python3
"""Porte « zéro terme » du build web Evora.

Le nom amont retiré ne doit plus apparaître dans le site servi, binaires compris, sauf
dans LICENSES/ où la licence impose de garder les mentions d'origine telles quelles.
Les copies .br et .gz sont sautées : leur fidélité au fichier d'origine est vérifiée
par la porte de précompression du bundle.

Usage :
  zero-terme.py check <dossier du site>   rouge si le terme reste hors de LICENSES/
  zero-terme.py selftest                  prouve que la porte sait rougir
"""
import contextlib
import io
import os
import re
import sys
import tempfile

TERM = b"c" + b"raft"
# Mots du langage courant sans rapport avec le nom retiré (tables Unicode, noms d'émojis...). Le terme doit
# finir la fenêtre lue (\Z) : un mot toléré plus tôt dans la fenêtre n'excuse pas l'occurrence jugée. « art »
# exige un mot de liaison : collé au terme, c'est le nom de l'équipe amont, jamais toléré.
UNRELATED = re.compile(rb"(?:air|space|hover|mine|star|war|witch|love|water|hand|arts? ?(?:&|and) ?)" + TERM + rb"\Z")
LICENSES = "LICENSES"
COMPRESSED = (".br", ".gz")


def scan(root):
    faults = []
    allowed = 0
    scanned = 0
    for current, dirs, files in os.walk(root):
        rel_dir = os.path.relpath(current, root)
        if rel_dir.split(os.sep)[0] == LICENSES:
            dirs[:] = []
            continue
        for name in files:
            if name.endswith(COMPRESSED):
                continue
            path = os.path.join(current, name)
            scanned += 1
            with open(path, "rb") as handle:
                lowered = handle.read().lower()
            start = 0
            while True:
                at = lowered.find(TERM, start)
                if at < 0:
                    break
                start = at + len(TERM)
                context = lowered[max(0, at - 24): at + 24]
                if UNRELATED.search(lowered[max(0, at - 12): at + len(TERM)]):
                    allowed += 1
                    continue
                faults.append((os.path.relpath(path, root), context))
    return faults, allowed, scanned


def check(root):
    faults, allowed, scanned = scan(root)
    if not scanned:
        print(f"::error::aucun fichier à juger dans {root!r} : la porte ne prouverait rien")
        return 1
    print(f"fichiers lus : {scanned} ; mots sans rapport tolérés : {allowed}")
    for rel, context in faults[:40]:
        print(f"::error::{rel} : {context!r}")
    print(f"occurrences en défaut : {len(faults)}")
    return 1 if faults else 0


def selftest():
    with tempfile.TemporaryDirectory() as top:
        clean = os.path.join(top, "propre")
        os.makedirs(os.path.join(clean, LICENSES))
        with open(os.path.join(clean, "a.js"), "wb") as handle:
            handle.write(b"console.log('air" + TERM + b"')")
        with open(os.path.join(clean, LICENSES, "NOTICE"), "wb") as handle:
            handle.write(b"Art" + TERM + b" Team")
        with open(os.path.join(clean, "b.js"), "wb") as handle:
            handle.write(b"arts and " + TERM + b"s, arts & " + TERM + b"s, hover" + TERM)
        clean_ok = not scan(clean)[0] and scan(clean)[1] == 4
        # Le nom de l'équipe amont, les noms de moteurs, les bibliothèques et le mot nu : tous rouges, un par un.
        traps = [b"art" + TERM, b"getart" + TERM, b"photo" + TERM, b"design" + TERM, b"vector" + TERM, TERM + b"-libs",
                 TERM + b"_ui", b"air" + TERM + b" p" + TERM, b"star" + TERM + b" " + TERM]
        trapped = os.path.join(top, "piege")
        os.makedirs(trapped)
        traps_ok = True
        for index, trap in enumerate(traps):
            piece = os.path.join(trapped, f"p{index}")
            os.makedirs(piece)
            with open(os.path.join(piece, "a.wasm"), "wb") as handle:
                handle.write(b"\0asm " + trap + b"\0")
            found = scan(piece)
            if len(found[0]) != 1:
                print(f"autotest : piège {index} vu {len(found[0])} fois au lieu d'une")
                traps_ok = False
        whole = len(scan(trapped)[0]) == len(traps)
        # Un dossier vide ou absent ne prouve rien : rouge, pas vert.
        empty = os.path.join(top, "vide")
        os.makedirs(empty)
        with contextlib.redirect_stdout(io.StringIO()):
            empty_ok = check(empty) == 1 and check(os.path.join(top, "absent")) == 1
        ok = clean_ok and traps_ok and whole and empty_ok
    print("autotest zéro terme :", "vert" if ok else "ROUGE")
    return 0 if ok else 1


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "selftest":
        sys.exit(selftest())
    if len(sys.argv) == 3 and sys.argv[1] == "check":
        sys.exit(check(sys.argv[2]))
    print(__doc__)
    sys.exit(2)
