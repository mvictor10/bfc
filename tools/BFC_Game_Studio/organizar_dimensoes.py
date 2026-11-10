#!/usr/bin/env python3
import os
import sys
import shutil
import cv2
from pathlib import Path

class Estatisticas:
    def __init__(self):
        self.personagens = 0
        self.sprites_renomeados = 0
        self.faces_renomeadas = 0
        self.erros = 0

def localizar_raiz_projeto() -> Path:
    """Localiza a raiz do projeto Brazil Fight Combat com base na posição do script."""
    script_path = Path(__file__).resolve()
    if script_path.parent.name == "BFC_Game_Studio" and script_path.parent.parent.name == "tools":
        return script_path.parent.parent.parent
    for pai in script_path.parents:
        if (pai / "chars").is_dir():
            return pai
    return Path.cwd()

def processar_dimensoes_personagem(pasta_personagem: Path, estatisticas: Estatisticas):
    """Analisa o tamanho das imagens e renomeia para sprite.png ou face.png."""
    pasta_backup = pasta_personagem / "backup_originais"
    arquivos = [p for p in pasta_personagem.iterdir() if p.is_file() and p.suffix.lower() == ".png"]

    for arquivo in arquivos:
        nome_atual = arquivo.name
        
        # Ignora se o arquivo já estiver com o nome correto definitivo
        if nome_atual in ["sprite.png", "face.png"]:
            continue

        try:
            # Carrega a imagem para ler as dimensões (fast read)
            img = cv2.imread(str(arquivo))
            if img is None:
                continue

            # OpenCV retorna (altura, largura, canais)
            altura, largura = img.shape[:2]
            novo_nome = None

            # Verifica as regras de resolução fornecidas
            if largura == 1152 and altura == 2048:
                novo_nome = "sprite.png"
            elif largura == 1600 and altura == 1600:
                novo_nome = "face.png"

            # Se a imagem bate com um dos padrões, faz o backup e renomeia
            if novo_nome:
                pasta_backup.mkdir(exist_ok=True)
                caminho_backup = pasta_backup / nome_atual
                
                # Salva uma cópia de segurança se ela ainda não existir
                if not caminho_backup.exists():
                    shutil.copy2(arquivo, caminho_backup)

                # Define o caminho final (ex: chars/nome_personagem/sprite.png)
                caminho_destino = pasta_personagem / novo_nome

                # Se o destino já existir por algum motivo, remove para evitar conflito
                if caminho_destino.exists():
                    caminho_destino.unlink()

                # Renomeia o arquivo original na pasta do personagem
                arquivo.rename(caminho_destino)
                
                if novo_nome == "sprite.png":
                    estatisticas.sprites_renomeados += 1
                else:
                    estatisticas.faces_renomeadas += 1
                    
                print(f"  [PADRONIZADO] {nome_atual} -> {novo_nome} ({largura}x{altura})")

        except Exception as e:
            estatisticas.erros += 1
            print(f"  [ERRO] Falha ao analisar dimensões de {nome_atual}: {e}")

def executar():
    raiz = localizar_raiz_projeto()
    pasta_chars = raiz / "chars"

    if not pasta_chars.is_dir():
        print(f"[FALHA] Pasta 'chars' não encontrada em: {pasta_chars}")
        sys.exit(1)

    personagens = sorted([d for d in pasta_chars.iterdir() if d.is_dir()])
    estatisticas = Estatisticas()

    print("====================================================")
    print("        BFC - PADRONIZAÇÃO DE RESOLUÇÃO (PYTHON)")
    print("====================================================")
    print(f"Projeto : {raiz}")
    print("Regras  : 1152x2048 -> sprite.png | 1600x1600 -> face.png")
    print("----------------------------------------------------")

    for pasta_personagem in personagens:
        estatisticas.personagens += 1
        print(f"\n[PERSONAGEM] {pasta_personagem.name}")
        processar_dimensoes_personagem(pasta_personagem, estatisticas)

    print("\n====================================================")
    print("                    RESUMO")
    print("====================================================")
    print(f"Personagens analisados : {estatisticas.personagens}")
    print(f"Sprites identificados  : {estatisticas.sprites_renomeados}")
    print(f"Faces identificadas    : {estatisticas.faces_renomeadas}")
    print(f"Erros encontrados      : {estatisticas.erros}")
    print("====================================================")
    print("Processamento de nomenclatura concluído.")

if __name__ == "__main__":
    try:
        executar()
    except Exception as e:
        print(f"\n[FALHA GLOBAL] {e}")
        sys.exit(1)
    if os.name == "nt":
        input("\nPressione ENTER para fechar...")
