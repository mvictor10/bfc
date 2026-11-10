#!/usr/bin/env python3
import os
import sys
import shutil
import cv2
import numpy as np
from pathlib import Path

class Estatisticas:
    def __init__(self):
        self.personagens = 0
        self.arquivos_analisados = 0
        self.convertidos = 0
        self.png_validos = 0
        self.erros = 0

def localizar_raiz_projeto() -> Path:
    """Localiza a raiz do projeto Brazil Fight Combat com base na posição do script."""
    script_path = Path(__file__).resolve()
    
    # Se estiver em tools/BFC_Game_Studio/converter_sprite.py, a raiz está 2 níveis acima
    if script_path.parent.name == "BFC_Game_Studio" and script_path.parent.parent.name == "tools":
        return script_path.parent.parent.parent
        
    # Fallback padrão: varre para cima procurando a pasta chars
    for pai in script_path.parents:
        if (pai / "chars").is_dir():
            return pai
            
    # Último recurso: diretório atual de execução
    return Path.cwd()

def remover_fundo_verde(caminho_imagem: Path) -> bool:
    """Remove o fundo verde gerando transparência real (Canal Alpha) limpa."""
    # Carrega a imagem original com 3 canais (BGR)
    img = cv2.imread(str(caminho_imagem))
    if img is None:
        return False

    # Converte para HSV para isolar o verde puro de forma cirúrgica
    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)

    # Intervalo exato do verde limão puro (#00ff00)
    verde_baixo = np.array([35, 50, 50])
    verde_alto = np.array([85, 255, 255])

    # Cria a máscara: onde for verde vira BRANCO (255), o resto PRETO (0)
    mascara_verde = cv2.inRange(hsv, verde_baixo, verde_alto)
    
    # Inverte: o personagem vira BRANCO (255) e o fundo vira PRETO (0)
    mascara_personagem = cv2.bitwise_not(mascara_verde)

    # CORREÇÃO: Aplica a máscara nos canais de cor para eliminar qualquer resíduo do fundo
    img_recortada = cv2.bitwise_and(img, img, mask=mascara_personagem)

    # Separa os canais de cor limpos
    b, g, r = cv2.split(img_recortada)
    
    # Junta B, G, R com a máscara do personagem no canal Alpha (Transparência real)
    img_transparente = cv2.merge([b, g, r, mascara_personagem])

    # Salva substituindo o arquivo por um PNG transparente legítimo
    cv2.imwrite(str(caminho_imagem), img_transparente)
    return True


def processar_personagem(pasta_personagem: Path, estatisticas: Estatisticas):
    """Processa todas as imagens do personagem criando backup local antes da conversão."""
    # Define e cria a pasta de backup individual por segurança
    pasta_backup = pasta_personagem / "backup_originais"
    
    arquivos = [p for p in pasta_personagem.iterdir() if p.is_file() and p.suffix.lower() == ".png"]
    
    for arquivo in arquivos:
        estatisticas.arquivos_analisados += 1
        nome_arq = arquivo.name
        
        try:
            # Garante que a pasta de backup existe se houver arquivos a processar
            pasta_backup.mkdir(exist_ok=True)
            caminho_backup = pasta_backup / nome_arq
            
            # Se o backup ainda não existir, faz a cópia do arquivo original
            if not caminho_backup.exists():
                shutil.copy2(arquivo, caminho_backup)
            
            # Executa a remoção do chroma key
            if remover_fundo_verde(arquivo):
                estatisticas.convertidos += 1
                print(f"  [CONVERTIDO] {nome_arq} (Fundo removido e salvo)")
            else:
                estatisticas.erros += 1
                print(f"  [ERRO] Não foi possível processar: {nome_arq}")
                
        except Exception as e:
            estatisticas.erros += 1
            print(f"  [ERRO] Falha ao processar {nome_arq}: {e}")

def executar():
    raiz = localizar_raiz_projeto()
    pasta_chars = raiz / "chars"

    if not pasta_chars.is_dir():
        print(f"[FALHA] Pasta 'chars' não encontrada em: {pasta_chars}")
        sys.exit(1)

    personagens = sorted([d for d in pasta_chars.iterdir() if d.is_dir()])
    estatisticas = Estatisticas()

    print("====================================================")
    print("          BRAZIL FIGHT COMBAT - SPRITE PNG (PYTHON)")
    print("====================================================")
    print(f"Projeto : {raiz}")
    print(f"Sprites : {pasta_chars}")
    print("----------------------------------------------------")

    for pasta_personagem in personagens:
        estatisticas.personagens += 1
        print(f"\n[PERSONAGEM] {pasta_personagem.name}")
        processar_personagem(pasta_personagem, estatisticas)

    print("\n====================================================")
    print("                    RESUMO")
    print("====================================================")
    print(f"Personagens analisados : {estatisticas.personagens}")
    print(f"Arquivos analisados    : {estatisticas.arquivos_analisados}")
    print(f"Convertidos com sucesso: {estatisticas.convertidos}")
    print(f"Erros encontrados      : {estatisticas.erros}")
    print("====================================================")
    print("Processamento concluído.")

if __name__ == "__main__":
    try:
        executar()
    except Exception as e:
        print(f"\n[FALHA GLOBAL] {e}")
        sys.exit(1)
        
    # Mantém o prompt aberto se rodar no Windows via duplo clique
    if os.name == "nt":
        input("\nPressione ENTER para fechar...")
