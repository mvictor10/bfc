(() => {
  'use strict';

  const $ = (id) => document.getElementById(id);
  const state = {
    view: 'dashboard', project: '', dashboard: null,
    characters: [], charId: '', character: null, characterDetail: null,
    stages: [], stageIndex: 0, moves: {}, rules: {}, stories: [], storyId: '',
    audio: { music: [], sfx: [], voice: [] }, audioTab: 'music',
    assetCategory: 'stages', assets: [], animationId: 'idle', animTimer: null, animFrame: 0,
    validation: null
  };

  const titles = {
    dashboard: 'Visão geral', characters: 'Personagens', stages: 'Fases e cenários',
    animations: 'Animações e sprites', audio: 'Áudio', stories: 'Histórias',
    moves: 'Golpes e dano', rules: 'Regras gerais', assets: 'Gerenciador de recursos',
    converter: 'Conversor de imagens', validator: 'Validação do projeto'
  };

  const escapeHtml = (value) => String(value ?? '').replace(/[&<>"']/g, ch => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]));
  const clone = (value) => JSON.parse(JSON.stringify(value));
  const fmtBytes = (n) => n < 1024 ? `${n} B` : n < 1048576 ? `${(n/1024).toFixed(1)} KB` : `${(n/1048576).toFixed(1)} MB`;
  const pathGet = (obj, path) => path.split('.').reduce((v, k) => v?.[k], obj);
  const pathSet = (obj, path, value) => {
    const parts = path.split('.'); let target = obj;
    for (const part of parts.slice(0, -1)) target = target[part] ??= {};
    target[parts.at(-1)] = value;
  };
  const toast = (message, kind = 'success') => {
    const node = document.createElement('div'); node.className = `toast ${kind}`; node.textContent = message;
    $('toast-region').appendChild(node); setTimeout(() => node.remove(), 4200);
  };
  const setStatus = (message) => { $('status-message').textContent = message; };
  const api = async (url, options = {}) => {
    const response = await fetch(url, options);
    let data;
    try { data = await response.json(); } catch { throw new Error(`Resposta inválida do servidor (${response.status}).`); }
    if (!response.ok || data.ok === false) throw new Error(data.error || `Erro HTTP ${response.status}`);
    return data;
  };
  const jsonOptions = (method, body) => ({ method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  const image = (url, cls = 'image-preview', alt = '') => url ? `<img class="${cls}" src="${escapeHtml(url)}" alt="${escapeHtml(alt)}" loading="lazy">` : `<div class="empty-state"><div class="empty-icon">▧</div><p>Imagem não configurada ou arquivo não encontrado.</p></div>`;
  const field = (path, label, value, type = 'text', extra = '') => `<div class="field"><label>${escapeHtml(label)}</label><input data-bind="${escapeHtml(path)}" type="${type}" value="${escapeHtml(value ?? '')}" ${extra}></div>`;
  const selectField = (path, label, value, options) => `<div class="field"><label>${escapeHtml(label)}</label><select data-bind="${escapeHtml(path)}">${options.map(o => `<option value="${escapeHtml(o.value)}" ${String(value ?? '') === String(o.value) ? 'selected' : ''}>${escapeHtml(o.label)}</option>`).join('')}</select></div>`;
  const pageHead = (eyebrow, title, desc, actions = '') => `<div class="page-head"><div><div class="eyebrow">${escapeHtml(eyebrow)}</div><h1>${escapeHtml(title)}</h1><p>${escapeHtml(desc)}</p></div><div class="page-actions">${actions}</div></div>`;
  const card = (content, cls = 'card card-pad') => `<section class="${cls}">${content}</section>`;

  function setProjectChrome() {
    const name = state.project ? state.project.split(/[\\/]/).filter(Boolean).at(-1) : 'Nenhum projeto aberto';
    $('project-mini-name').textContent = name;
    $('project-label').textContent = state.project || 'Aguardando projeto';
    $('project-mini').title = state.project;
  }

  async function openProject(path) {
    const data = await api('/api/project/open', jsonOptions('POST', { path }));
    state.project = data.project;
    state.characters = []; state.charId = ''; state.character = null; state.characterDetail = null;
    state.stages = []; state.stageIndex = 0; state.moves = {}; state.rules = {}; state.stories = []; state.storyId = '';
    state.audio = {music:[],sfx:[],voice:[]}; state.assets = [];
    setProjectChrome();
    toast(`Projeto aberto: ${data.name}`);
    await loadView('dashboard');
  }

  async function initialize() {
    try {
      const status = await api('/api/status');
      if (status.opened) state.project = status.project;
      setProjectChrome();
      await loadView('dashboard');
    } catch (err) {
      toast(err.message, 'error');
      await renderDashboardEmpty();
    }
  }

  async function loadView(view) {
    state.view = view;
    if (state.animTimer) { clearInterval(state.animTimer); state.animTimer = null; }
    document.querySelectorAll('.nav-item').forEach(button => button.classList.toggle('active', button.dataset.view === view));
    $('current-section').textContent = titles[view] || view;
    setStatus(`Módulo: ${titles[view] || view}`);
    $('view-root').innerHTML = `<div class="loading-card"><div class="spinner"></div><p>Carregando ${escapeHtml(titles[view] || view)}...</p></div>`;
    if (!state.project && view !== 'dashboard') {
      await renderDashboardEmpty(); return;
    }
    try {
      switch (view) {
        case 'dashboard': return await renderDashboard();
        case 'characters': return await renderCharacters();
        case 'stages': return await renderStages();
        case 'animations': return await renderAnimations();
        case 'audio': return await renderAudio();
        case 'stories': return await renderStories();
        case 'moves': return await renderMoves();
        case 'rules': return await renderRules();
        case 'assets': return await renderAssets();
        case 'converter': return renderConverter();
        case 'validator': return await renderValidator();
        default: return await renderDashboard();
      }
    } catch (err) {
      $('view-root').innerHTML = `<div class="notice error">${escapeHtml(err.message)}</div>`;
      toast(err.message, 'error');
    }
  }

  async function renderDashboardEmpty() {
    $('view-root').innerHTML = `${pageHead('BRAZIL FIGHT COMBAT · TOOLING', 'Abra seu projeto', 'Selecione a pasta que contém Cargo.toml e chars/chars.json. O Studio editará os arquivos reais do projeto, mantendo o motor de combate em Rust.')}
      <div class="dashboard-hero"><div><div class="eyebrow">BFC GAME STUDIO</div><h1>Seu jogo, em um editor visual.</h1><p>Gerencie personagens, cenários, sprites, animações, áudio, golpes, regras e histórias a partir de uma única interface local.</p></div><div class="hero-art">BFC</div></div>
      ${card(`<div class="section-title"><div><h2>Abrir pasta do projeto</h2><p>Informe o caminho completo da raiz do Brazil Fight Combat no Windows.</p></div></div><div class="form-grid"><div class="field wide"><label>Caminho do projeto</label><input id="project-path" class="text-input" placeholder="C:\\Users\\...\\bfc"></div></div><div class="toolbar" style="margin-top:14px"><div class="muted small">Obrigatório: Cargo.toml e chars/chars.json</div><button class="btn btn-primary" id="connect-project">Conectar projeto →</button></div>`)}
      <div class="grid grid-3" style="margin-top:16px">${['Editor de personagens','Editor de fases','Estúdio de animações','Gerenciador de áudio','Configuração de golpes','Validador de recursos'].map((x,i)=>card(`<div class="metric-icon">${['♟','▧','▤','♫','⚔','✓'][i]}</div><h2 style="margin:12px 0 5px">${x}</h2><p class="muted small">Ferramentas integradas aos arquivos do seu projeto.</p>`)).join('')}</div>`;
    $('connect-project')?.addEventListener('click', async () => {
      const path = $('project-path').value.trim(); if (!path) return toast('Informe o caminho da pasta do projeto.', 'error');
      try { await openProject(path); } catch (err) { toast(err.message, 'error'); }
    });
  }

  async function renderDashboard() {
    if (!state.project) return renderDashboardEmpty();
    const d = await api('/api/dashboard'); state.dashboard = d;
    $('view-root').innerHTML = `${pageHead('BFC GAME STUDIO', 'Painel do projeto', 'Visão geral dos recursos encontrados e acesso rápido aos módulos de edição.', `<button class="btn btn-secondary" id="refresh-dashboard">Atualizar</button><button class="btn btn-secondary" id="build-game">⚒ Compilar</button><button class="btn btn-primary" id="launch-game">▶ Executar jogo</button>`)}
      <div class="dashboard-hero"><div><div class="eyebrow">PROJETO CONECTADO</div><h1>${escapeHtml(state.project.split(/[\\/]/).filter(Boolean).at(-1))}</h1><p>${escapeHtml(state.project)}<br>O Studio grava alterações nos arquivos de configuração e recursos. Faça backup antes de mudanças grandes.</p><div class="page-actions"><button class="btn btn-primary" data-go="characters">Editar personagens</button><button class="btn btn-secondary" data-go="stages">Editar fases</button></div></div><div class="hero-art">⚔</div></div>
      <div class="grid grid-4">
        ${metric('Personagens', d.characters, 'Registrados no elenco', '♟')}
        ${metric('Fases', d.stages, 'Cadastradas em stages.ini', '▧')}
        ${metric('Sprites', d.sprites, 'Imagens nos diretórios do elenco', '▤')}
        ${metric('Imagens de fase', d.stage_images, 'Arquivos em assets/images/stages', '▧')}
        ${metric('Trilhas sonoras', d.music, 'Biblioteca de música', '♫')}
        ${metric('Efeitos sonoros', d.sfx, 'Biblioteca de SFX', '♪')}
        ${metric('Vozes', d.voices, 'Arquivos de voz', '◖')}
        ${metric('Resolução lógica', '960×540', 'Definida pelo motor Rust', '▣')}
      </div>
      <div id="build-result" class="hidden card card-pad" style="margin-top:16px"><div class="section-title"><div><h2>Resultado da compilação</h2><p id="build-status">Aguardando.</p></div></div><pre id="build-log" style="white-space:pre-wrap;overflow-wrap:anywhere;max-height:280px;overflow:auto;font-size:10px;color:#b7c5d8"></pre></div>
      <div class="grid grid-2" style="margin-top:17px">
        ${card(`<div class="section-title"><div><h2>Acesso rápido</h2><p>Áreas principais de produção.</p></div></div><div class="grid grid-2">${quick('characters','♟','Personagens')}${quick('stages','▧','Fases')}${quick('animations','▤','Animações')}${quick('audio','♫','Áudio')}${quick('moves','⚔','Golpes')}${quick('rules','⚙','Regras')}</div>`)}
        ${card(`<div class="section-title"><div><h2>Ferramentas de projeto</h2><p>Verifique e normalize os recursos.</p></div></div><div class="notice">O conversor de imagens analisa o formato pelo conteúdo do arquivo. Sprites e cenários são convertidos para PNG sem aplicar chroma key aos cenários.</div><div class="toolbar" style="margin-top:13px"><button class="btn btn-secondary" data-go="converter">Conversor de imagens</button><button class="btn btn-secondary" data-go="validator">Validar projeto</button></div><div class="muted small">O motor Rust permanece responsável pela simulação, colisões, IA e renderização do jogo.</div>`)}
      </div>`;
    $('refresh-dashboard').addEventListener('click', () => loadView('dashboard'));
    $('launch-game')?.addEventListener('click', async () => { try { const res = await api('/api/game/launch', {method:'POST'}); toast(res.message); } catch (err) { toast(err.message, 'error'); } });
    $('build-game')?.addEventListener('click', async () => {
      const button = $('build-game'); button.disabled = true; button.textContent = 'Compilando...';
      try {
        const result = await api('/api/game/build', {method:'POST'});
        $('build-result').classList.remove('hidden');
        $('build-status').textContent = result.build_ok ? 'Compilação concluída.' : `Falha na compilação (código ${result.returncode}).`;
        $('build-log').textContent = [result.stdout, result.stderr].filter(Boolean).join('\n');
        toast(result.build_ok ? 'Jogo compilado em release.' : 'A compilação falhou. Consulte o log.', result.build_ok ? 'success' : 'error');
      } catch (err) { toast(err.message, 'error'); }
      finally { button.disabled = false; button.textContent = '⚒ Compilar'; }
    });
    document.querySelectorAll('[data-go]').forEach(b => b.addEventListener('click', () => loadView(b.dataset.go)));
  }
  function metric(label, value, note, icon) { return `<div class="card metric"><div class="metric-top"><span>${label}</span><span class="metric-icon">${icon}</span></div><strong>${escapeHtml(value)}</strong><small>${note}</small></div>`; }
  function quick(view, icon, label) { return `<button class="nav-item" style="background:#172638;border-color:#293d51" data-go="${view}"><span>${icon}</span>${label}</button>`; }

  async function renderCharacters() {
    state.characters = (await api('/api/characters')).items;
    if (!state.stages.length) state.stages = (await api('/api/stages')).items;
    if (!state.charId || !state.characters.some(c => c.id === state.charId)) state.charId = state.characters[0]?.id || '';
    state.characterDetail = state.charId ? await api(`/api/characters/${encodeURIComponent(state.charId)}`) : null;
    state.character = state.characterDetail?.config || null;
    const list = state.characters.map(c => `<button class="list-row ${c.id === state.charId ? 'selected' : ''}" data-char="${escapeHtml(c.id)}">${c.face_url ? `<img class="list-thumb" src="${escapeHtml(c.face_url)}">` : `<div class="list-thumb initial">${escapeHtml(c.name.slice(0,1))}</div>`}<span style="min-width:0"><strong>${escapeHtml(c.name)}</strong><small>${escapeHtml(c.id)} · HP ${escapeHtml(c.hp)} · ${c.image_count} imagens</small></span></button>`).join('');
    let editor = `<div class="empty-state"><div class="empty-icon">♟</div><h2>Elenco vazio</h2><p>Crie um personagem para começar. A nova entrada será adicionada a chars/chars.json.</p><button class="btn btn-primary" id="create-character">Criar personagem</button></div>`;
    if (state.character) editor = characterEditor(state.character, state.characterDetail);
    $('view-root').innerHTML = `${pageHead('CONTEÚDO DO JOGO', 'Personagens', 'Edite atributos, artes, associação de fase e parâmetros lidos de chars/<id>/character.json.', `<button class="btn btn-primary" id="create-character">+ Novo personagem</button>`)}
      <div class="editor-layout"><section class="card list-card"><div class="list-header"><div class="section-title"><div><h2>Elenco (${state.characters.length})</h2><p>chars/chars.json</p></div></div><input class="text-input" id="character-search" placeholder="Filtrar personagens..."></div><div class="list-scroll" id="character-list">${list}</div></section><div class="editor-main">${editor}</div></div>`;
    document.querySelectorAll('[data-char]').forEach(b => b.addEventListener('click', () => { state.charId = b.dataset.char; renderCharacters(); }));
    $('create-character')?.addEventListener('click', createCharacter);
    $('character-search')?.addEventListener('input', e => document.querySelectorAll('[data-char]').forEach(b => b.classList.toggle('hidden', !b.textContent.toLowerCase().includes(e.target.value.toLowerCase()))));
    $('save-character')?.addEventListener('click', saveCharacterForm);
    $('save-character-advanced')?.addEventListener('click', saveCharacterAdvanced);
    $('delete-character')?.addEventListener('click', removeCharacter);
    $('upload-character-file')?.addEventListener('click', () => uploadSelected('character:' + state.charId, $('character-file')));
  }

  function characterEditor(cfg, detail) {
    const stageOptions = [{value:'',label:'Nenhuma preferência'}, ...state.stages.map(s => ({value:s.id,label:s.name}))];
    const animationRows = Object.entries(cfg.animations || {}).map(([name, a]) => `<tr><td>${escapeHtml(name)}</td><td>${escapeHtml(a.prefix || name)}</td><td>${escapeHtml(a.fps ?? '—')}</td><td>${escapeHtml(a.mode || '—')}</td></tr>`).join('');
    const art = cfg.art || {};
    const sprite = detail.frames.find(f => f.name === (art.sprite || 'sprite.png'));
    const face = detail.frames.find(f => f.name === (art.face || 'face.png'));
    return `<div class="card card-pad"><div class="toolbar"><div><div class="eyebrow">CHARACTER CONFIGURATION</div><h2 style="font-size:19px;margin:5px 0">${escapeHtml(cfg.name || detail.id)}</h2><div class="muted small">chars/${escapeHtml(detail.id)}/character.json</div></div><div class="toolbar-right"><span class="tag good">Registrado no elenco</span><button class="btn btn-danger btn-small" id="delete-character">Remover do elenco</button></div></div>
      <div class="tabs"><span class="tab active">Dados gerais</span><span class="tab">Visual</span><span class="tab">Animações</span><span class="tab">Avançado</span></div>
      <div class="section-title"><div><h2>Identidade e atributos</h2><p>Campos reconhecidos pelo carregador Rust.</p></div></div>
      <div class="form-grid cols-3">
        ${field('name','Nome',cfg.name)}${field('stats.hp','Pontos de vida (HP)',cfg.stats?.hp ?? 200,'number','min="1" max="5000" step="1"')}${field('stats.speed','Velocidade',cfg.stats?.speed ?? 4.5,'number','min="1" max="12" step="0.1"')}
        ${field('stats.power','Multiplicador de força',cfg.stats?.power ?? 1,'number','min="0.1" max="5" step="0.1"')}${field('size','Tamanho do personagem',cfg.size ?? 300,'number','min="32" max="800"')}${field('body.w','Largura corporal',cfg.body?.w ?? 70,'number','min="10" max="300" step="1"')}${field('body.h','Altura corporal',cfg.body?.h ?? 230,'number','min="20" max="400" step="1"')}${selectField('facing','Orientação inicial',cfg.facing || 'right',[{value:'right',label:'Direita'},{value:'left',label:'Esquerda'}])}
        ${selectField('stage','Fase associada',cfg.stage || '',stageOptions)}${field('music','ID da música (opcional)',cfg.music || '')}${field('voice','ID da voz',cfg.voice || detail.id)}
      </div>
      <div class="section-title" style="margin-top:20px"><div><h2>Arte e enquadramento</h2><p>Arquivos relativos à pasta do personagem.</p></div></div>
      <div class="grid grid-2"><div>${sprite ? image(sprite.url,'image-preview','Sprite principal') : ''}${field('art.sprite','Arquivo do sprite',art.sprite || 'sprite.png')}${face ? image(face.url,'image-preview','Retrato') : ''}${field('art.face','Arquivo do retrato',art.face || 'face.png')}</div><div class="form-grid">
        ${field('anchor.x','Anchor X',cfg.anchor?.x ?? 0.5,'number','min="0" max="1" step="0.01"')}${field('anchor.y','Anchor Y',cfg.anchor?.y ?? 0.92,'number','min="0" max="1" step="0.01"')}
        ${field('chroma.color','Cor do chroma key',cfg.chroma?.color || '#00ff00')}${field('chroma.tolerance','Tolerância do chroma',cfg.chroma?.tolerance ?? 90,'number','min="0" max="255"')}
        ${field('chroma.softness','Suavização do chroma',cfg.chroma?.softness ?? 60,'number','min="0" max="255"')}${field('chroma.maxHeight','Altura máxima dos frames',cfg.chroma?.maxHeight ?? 560,'number','min="128" max="1200"')}
      </div></div>
      <div class="section-title" style="margin-top:20px"><div><h2>Rig / proporções</h2><p>Aplicado quando há configuração de rig no personagem.</p></div></div>
      <div class="form-grid cols-3">${field('rig.neck','Pescoço',cfg.rig?.neck ?? 0.24,'number','step="0.001" min="0" max="1"')}${field('rig.hip','Quadril',cfg.rig?.hip ?? 0.57,'number','step="0.001" min="0" max="1"')}${field('rig.split','Divisão corporal',cfg.rig?.split ?? 0.5,'number','step="0.001" min="0" max="1"')}
        <div class="field"><label>Espelhar rig</label><select data-bind="rig.flip"><option value="0" ${!cfg.rig?.flip ? 'selected':''}>Não</option><option value="1" ${cfg.rig?.flip ? 'selected':''}>Sim</option></select></div>
        <div class="field"><label>Rig com saia</label><select data-bind="rig.skirt"><option value="0" ${!cfg.rig?.skirt ? 'selected':''}>Não</option><option value="1" ${cfg.rig?.skirt ? 'selected':''}>Sim</option></select></div>
      </div>
      <div class="section-title" style="margin-top:20px"><div><h2>Animações configuradas</h2><p>O editor detalhado fica na seção Animações e sprites.</p></div><button class="btn btn-secondary btn-small" data-go="animations">Abrir Animation Studio</button></div>
      <div class="table-wrap"><table class="table"><thead><tr><th>Estado</th><th>Prefixo</th><th>FPS</th><th>Modo</th></tr></thead><tbody>${animationRows || '<tr><td colspan="4">Nenhuma animação cadastrada.</td></tr>'}</tbody></table></div>
      <details style="margin-top:18px"><summary>Dados avançados (JSON) — efeitos, animações e overrides de golpes</summary><div class="details-content"><p class="muted small">Edite o objeto completo do personagem. Campos desconhecidos são preservados. JSON inválido não será gravado.</p><textarea id="character-advanced" class="field code-area" spellcheck="false">${escapeHtml(JSON.stringify({animations:cfg.animations || {},effects:cfg.effects || {},moves:cfg.moves || {},voices:cfg.voices || {},sheet:cfg.sheet || {}},null,2))}</textarea><button class="btn btn-secondary" id="save-character-advanced" style="margin-top:10px">Salvar dados avançados</button></div></details>
      <div class="section-title" style="margin-top:18px"><div><h2>Arquivos do personagem (${detail.frames.length})</h2><p>Imagens reconhecidas na pasta chars/${escapeHtml(detail.id)}.</p></div></div>
      <div class="asset-grid">${detail.frames.slice(0,16).map(f=>`<div class="asset-card"><img src="${escapeHtml(f.url)}" alt="${escapeHtml(f.name)}"><strong>${escapeHtml(f.name)}</strong><small>${fmtBytes(f.size)}</small></div>`).join('')}</div>
      <div class="dropzone" style="margin-top:15px">Adicionar sprite, retrato ou frame à pasta do personagem<input id="character-file" type="file" accept="image/*,.bmp"><button class="btn btn-secondary btn-small" id="upload-character-file" style="margin-top:9px">Enviar arquivo</button></div>
      <div class="toolbar" style="margin-top:17px"><span class="muted small">As alterações atualizam character.json. Uma cópia .bak é criada na primeira gravação.</span><button class="btn btn-primary" id="save-character">Salvar personagem</button></div>
    </div>`;
  }

  async function createCharacter() {
    const id = prompt('ID do novo personagem (ex.: novo-lutador):'); if (!id) return;
    const name = prompt('Nome de exibição:', id.replaceAll('-', ' ')) || id;
    try { const res = await api('/api/characters', jsonOptions('POST', {id, name})); state.charId = res.id; toast('Personagem criado. Adicione os sprites e ajuste as animações.'); await renderCharacters(); }
    catch (err) { toast(err.message, 'error'); }
  }

  function collectBoundFields(source) {
    const result = clone(source);
    document.querySelectorAll('[data-bind]').forEach(el => {
      const path = el.dataset.bind;
      let value = el.value;
      if (el.type === 'number') value = value === '' ? 0 : Number(value);
      if (path === 'rig.flip' || path === 'rig.skirt') value = value === '1';
      pathSet(result, path, value);
    });
    return result;
  }

  async function saveCharacterForm() {
    try {
      const result = collectBoundFields(state.character);
      await api(`/api/characters/${encodeURIComponent(state.charId)}`, jsonOptions('PUT', result));
      toast('Personagem salvo com sucesso.'); await renderCharacters();
    } catch (err) { toast(err.message, 'error'); }
  }

  async function saveCharacterAdvanced() {
    try {
      const advanced = JSON.parse($('character-advanced').value);
      const result = collectBoundFields(state.character);
      for (const key of ['animations','effects','moves','voices','sheet']) {
        if (Object.prototype.hasOwnProperty.call(advanced, key)) result[key] = advanced[key];
      }
      await api(`/api/characters/${encodeURIComponent(state.charId)}`, jsonOptions('PUT', result));
      toast('Dados avançados salvos.'); await renderCharacters();
    } catch (err) { toast(`Não foi possível salvar: ${err.message}`, 'error'); }
  }

  async function removeCharacter() {
    if (!confirm(`Remover ${state.character.name} do elenco? A pasta e os sprites serão preservados.`)) return;
    try { await api(`/api/characters/${encodeURIComponent(state.charId)}`, {method:'DELETE'}); state.charId=''; toast('Personagem removido do elenco; recursos preservados.'); await renderCharacters(); }
    catch (err) { toast(err.message, 'error'); }
  }

  async function uploadSelected(category, input) {
    const file = input?.files?.[0]; if (!file) return toast('Selecione um arquivo primeiro.', 'error');
    const form = new FormData(); form.append('category', category); form.append('file', file);
    try { const res = await api('/api/upload', {method:'POST', body:form}); toast(`Arquivo enviado: ${res.path}`); if (state.view === 'characters') await renderCharacters(); else if (state.view === 'stages') await renderStages(); else if (state.view === 'assets') await renderAssets(); else if (state.view === 'audio') await renderAudio(); }
    catch (err) { toast(err.message, 'error'); }
  }

  async function renderStages() {
    state.stages = (await api('/api/stages')).items;
    if (state.stageIndex >= state.stages.length) state.stageIndex = Math.max(0, state.stages.length - 1);
    const stage = state.stages[state.stageIndex];
    const list = state.stages.map((s,i)=>`<button class="list-row ${i===state.stageIndex?'selected':''}" data-stage-index="${i}">${s.image_url?`<img class="list-thumb" src="${escapeHtml(s.image_url)}">`:'<div class="list-thumb initial">▧</div>'}<span><strong>${escapeHtml(s.name)}</strong><small>${escapeHtml(s.id)} · ${s.image_exists?'imagem OK':'sem imagem'}</small></span></button>`).join('');
    let editor = `<div class="empty-state"><h2>Nenhuma fase</h2><p>Crie a primeira fase para começar.</p><button class="btn btn-primary" id="new-stage">+ Criar fase</button></div>`;
    if (stage) editor = `<div class="card card-pad"><div class="toolbar"><div><div class="eyebrow">STAGE DESIGNER</div><h2 style="font-size:19px;margin:5px 0">${escapeHtml(stage.name)}</h2><div class="muted small">data/stages.ini · stage ${state.stageIndex+1}</div></div><div class="toolbar-right"><span class="tag ${stage.image_exists?'good':'warn'}">${stage.image_exists?'Imagem localizada':'Imagem ausente'}</span></div></div>
      ${stage.image_url?`<div class="stage-preview-wrap"><img class="stage-preview" src="${escapeHtml(stage.image_url)}" alt="${escapeHtml(stage.name)}"><div class="ground-guide" style="top:${Math.max(0,Math.min(100,Number(stage.ground)/540*100))}%"><span>GROUND ${escapeHtml(stage.ground)}</span></div></div>`:'<div class="notice warning">Esta fase ainda não tem uma imagem válida associada.</div>'}
      <div class="form-grid" style="margin-top:16px">${field('stage.id','ID',stage.id)}${field('stage.name','Nome',stage.name)}${field('stage.image','Caminho da imagem',stage.image)}${field('stage.ground','Altura do chão (0–540)',stage.ground,'number','min="0" max="540" step="1"')}${field('stage.music','ID da trilha sonora (ex.: tema_cidade)',stage.music==='null'?'':stage.music)}</div>
      <div class="dropzone" style="margin-top:14px">Enviar uma nova imagem para assets/images/stages/<input id="stage-image-file" type="file" accept="image/*"><button class="btn btn-secondary btn-small" id="upload-stage-image" style="margin-top:9px">Enviar imagem da fase</button></div>
      <div class="toolbar" style="margin-top:17px"><span class="muted small">O motor usa o caminho relativo, ID, nome, música e ground desta seção.</span><div class="toolbar-right"><button class="btn btn-danger" id="delete-stage">Remover fase</button><button class="btn btn-secondary" id="add-stage">+ Nova fase</button><button class="btn btn-primary" id="save-stages">Salvar todas as fases</button></div></div>
    </div>`;
    $('view-root').innerHTML = `${pageHead('WORLD / LEVEL EDITOR', 'Fases e cenários', 'Gerencie os 26 cenários e associe imagem, música e posição do chão. As configurações são gravadas em data/stages.ini.', `<button class="btn btn-primary" id="add-stage-top">+ Nova fase</button>`)}<div class="editor-layout"><section class="card list-card"><div class="list-header"><h2>Biblioteca de fases (${state.stages.length})</h2><p class="muted small">assets/images/stages/</p></div><div class="list-scroll">${list}</div></section><div class="editor-main">${editor}</div></div>`;
    document.querySelectorAll('[data-stage-index]').forEach(b=>b.addEventListener('click',async()=>{stageCollect();try{await api('/api/stages',jsonOptions('PUT',{items:state.stages}));state.stageIndex=Number(b.dataset.stageIndex);await renderStages();}catch(err){toast(err.message,'error');}}));
    $('add-stage')?.addEventListener('click',addStage); $('add-stage-top')?.addEventListener('click',addStage); $('new-stage')?.addEventListener('click',addStage);
    $('delete-stage')?.addEventListener('click',deleteStage);
    $('save-stages')?.addEventListener('click',saveStages);
    $('upload-stage-image')?.addEventListener('click',async()=>{
      const before = $('stage-image-file').files?.[0]; if(!before)return toast('Selecione uma imagem.', 'error');
      const form=new FormData();form.append('category','stages');form.append('file',before);
      try{const res=await api('/api/upload',{method:'POST',body:form});stageCollect();state.stages[state.stageIndex].image=res.path;await api('/api/stages',jsonOptions('PUT',{items:state.stages}));toast('Imagem enviada e associada à fase.');await renderStages();}catch(err){toast(err.message,'error');}
    });
  }
  function stageCollect() {
    if (!state.stages[state.stageIndex]) return;
    document.querySelectorAll('[data-bind^="stage."]').forEach(el=>{
      const key=el.dataset.bind.slice(6);let value=el.value;if(el.type==='number')value=Number(value);if(key==='music'&&!value)value='null';state.stages[state.stageIndex][key]=value;
    });
  }
  async function addStage() {
    const id=prompt('ID da nova fase (ex.: nova-cidade):');if(!id)return;
    stageCollect();
    try{await api('/api/stages',jsonOptions('PUT',{items:state.stages}));await api('/api/stages',jsonOptions('POST',{id,name:id.replaceAll('-',' ')}));state.stages=(await api('/api/stages')).items;state.stageIndex=state.stages.length-1;toast('Fase criada.');renderStages();}catch(err){toast(err.message,'error');}
  }
  async function saveStages() { stageCollect();try{await api('/api/stages',jsonOptions('PUT',{items:state.stages}));toast('Fases salvas em data/stages.ini.');await renderStages();}catch(err){toast(err.message,'error');} }
  async function deleteStage() {
    const stage=state.stages[state.stageIndex]; if(!stage)return;
    if(!confirm(`Remover a fase ${stage.name} do stages.ini? A imagem continuará na pasta de recursos.`))return;
    stageCollect(); state.stages.splice(state.stageIndex,1); state.stageIndex=Math.max(0,state.stageIndex-1);
    try{await api('/api/stages',jsonOptions('PUT',{items:state.stages}));toast('Fase removida do cadastro. O arquivo de imagem foi preservado.');await renderStages();}catch(err){toast(err.message,'error');}
  }

  async function renderAnimations() {
    if (!state.characters.length) state.characters=(await api('/api/characters')).items;
    if (!state.charId || !state.characters.some(c=>c.id===state.charId)) state.charId=state.characters[0]?.id||'';
    if (!state.charId) { $('view-root').innerHTML=pageHead('ANIMATION WORKSPACE','Animações e sprites','Escolha um personagem.')+'<div class="empty-state">Nenhum personagem registrado.</div>';return; }
    state.characterDetail=await api(`/api/characters/${encodeURIComponent(state.charId)}`);state.character=state.characterDetail.config;
    const anims=state.character.animations||{};const names=Object.keys(anims);if(!names.includes(state.animationId))state.animationId=names[0]||'idle';
    const anim=anims[state.animationId]||{prefix:state.animationId,fps:8,mode:'loop'};
    const frames=state.characterDetail.frames.filter(f=>f.name.toLowerCase().startsWith(String(anim.prefix||state.animationId).toLowerCase()+'_')).sort((a,b)=>a.name.localeCompare(b.name,undefined,{numeric:true,sensitivity:'base'}));
    const selectedCharOptions=state.characters.map(c=>({value:c.id,label:c.name}));
    $('view-root').innerHTML=`${pageHead('ANIMATION WORKSPACE','Animações e sprites','Configure prefixo, FPS e modo de reprodução. Pré-visualização baseada nos frames encontrados na pasta do personagem.',`<button class="btn btn-secondary" data-go="characters">Abrir cadastro</button>`)}
      <div class="grid grid-2">
        <div class="card card-pad"><div class="section-title"><div><h2>Sequência de animação</h2><p>${escapeHtml(state.characterDetail.directory)}</p></div><span class="tag">${frames.length} frames</span></div>
          <div class="form-grid">${selectField('anim.character','Personagem',state.charId,selectedCharOptions)}${selectField('anim.name','Animação',state.animationId,names.map(n=>({value:n,label:n})))}</div>
          <div class="form-grid" style="margin-top:13px">${field('anim.prefix','Prefixo dos arquivos',anim.prefix||state.animationId)}${field('anim.fps','FPS',anim.fps??8,'number','min="1" max="60" step="1"')}${selectField('anim.mode','Modo',anim.mode||'loop',['loop','once','progress','vy'].map(v=>({value:v,label:v})))}</div>
          <div class="toolbar" style="margin-top:14px"><button class="btn btn-secondary" id="play-animation">▶ Reproduzir</button><button class="btn btn-secondary" id="stop-animation">■ Parar</button><button class="btn btn-primary" id="save-animation">Salvar animação</button></div>
        </div>
        <div class="card card-pad"><div class="section-title"><div><h2>Preview de frames</h2><p>Pré-visualização de arte; não simula a física do motor.</p></div></div><div style="display:grid;place-items:center;min-height:250px">${frames[0]?`<img id="animation-preview" class="character-preview" src="${escapeHtml(frames[0].url)}" alt="Preview">`:'<div class="empty-state">Nenhum frame com este prefixo.</div>'}</div><div id="animation-counter" class="muted small" style="margin-top:8px">${frames.length?`Frame 1 / ${frames.length}`:'Sem frames'}</div></div>
      </div>
      <div class="card card-pad" style="margin-top:16px"><div class="section-title"><div><h2>Frames encontrados</h2><p>Arquivos de imagem associados ao prefixo.</p></div></div><div class="asset-grid">${frames.map((f,i)=>`<div class="asset-card"><img src="${escapeHtml(f.url)}"><strong>${escapeHtml(f.name)}</strong><small>Frame ${i+1} · ${fmtBytes(f.size)}</small></div>`).join('')||'<div class="muted small">Adicione imagens como idle_01.png, idle_02.png, walk_01.png etc.</div>'}</div><div class="dropzone" style="margin-top:14px">Adicionar novos frames à pasta chars/${escapeHtml(state.charId)}<input type="file" id="animation-frame-upload" accept="image/*" multiple><button class="btn btn-secondary btn-small" id="upload-animation-frames" style="margin-top:8px">Enviar frames</button></div></div>`;
    $('upload-animation-frames')?.addEventListener('click',async()=>{const files=[...$('animation-frame-upload').files];if(!files.length)return toast('Selecione um ou mais frames.','error');try{for(const file of files){const form=new FormData();form.append('category','character:'+state.charId);form.append('file',file);await api('/api/upload',{method:'POST',body:form});}toast(`${files.length} arquivo(s) enviado(s).`);await renderAnimations();}catch(err){toast(err.message,'error');}});
    const charSel=document.querySelector('[data-bind="anim.character"]');charSel.addEventListener('change',()=>{state.charId=charSel.value;renderAnimations();});
    const nameSel=document.querySelector('[data-bind="anim.name"]');nameSel.addEventListener('change',()=>{state.animationId=nameSel.value;renderAnimations();});
    $('play-animation')?.addEventListener('click',()=>playFrames(frames,Number(document.querySelector('[data-bind="anim.fps"]')?.value||8)));
    $('stop-animation')?.addEventListener('click',()=>{if(state.animTimer)clearInterval(state.animTimer);state.animTimer=null;});
    $('save-animation')?.addEventListener('click',async()=>{
      const updated=clone(state.character);const prefix=document.querySelector('[data-bind="anim.prefix"]').value.trim();const fps=Number(document.querySelector('[data-bind="anim.fps"]').value);const mode=document.querySelector('[data-bind="anim.mode"]').value;
      if(!prefix||!Number.isFinite(fps)||fps<1||fps>60)return toast('Prefixo ou FPS inválido.','error');updated.animations??={};updated.animations[state.animationId]={...(updated.animations[state.animationId]||{}),prefix,fps,mode};
      try{await api(`/api/characters/${encodeURIComponent(state.charId)}`,jsonOptions('PUT',updated));toast('Configuração da animação salva.');await renderAnimations();}catch(err){toast(err.message,'error');}
    });
  }
  function playFrames(frames,fps){if(!frames.length)return toast('Não há frames para reproduzir.','error');if(state.animTimer)clearInterval(state.animTimer);state.animFrame=0;const img=$('animation-preview');const label=$('animation-counter');const delay=1000/Math.max(1,Math.min(60,fps||8));state.animTimer=setInterval(()=>{state.animFrame=(state.animFrame+1)%frames.length;if(img)img.src=frames[state.animFrame].url;if(label)label.textContent=`Frame ${state.animFrame+1} / ${frames.length}`;},delay);}

  async function renderAudio() {
    if (!state.characters.length) state.characters=(await api('/api/characters')).items;
    state.audio=await api('/api/audio');const entries=state.audio[state.audioTab]||[];
    const tabs=[['music','Trilhas sonoras'],['sfx','Efeitos sonoros'],['voice','Vozes']];
    $('view-root').innerHTML=`${pageHead('AUDIO ASSET MANAGER','Áudio','Biblioteca de músicas, efeitos e vozes. Reproduza os arquivos no navegador para conferir o conteúdo.',`<button class="btn btn-secondary" data-go="assets">Gerenciar arquivos</button>`)}
      <div class="tabs">${tabs.map(([key,label])=>`<button class="tab ${state.audioTab===key?'active':''}" data-audio-tab="${key}">${label} <span class="tag">${state.audio[key]?.length||0}</span></button>`).join('')}</div>
      <div class="card card-pad"><div class="toolbar"><div><h2>${tabs.find(t=>t[0]===state.audioTab)[1]}</h2><p class="muted small">${state.audioTab==='music'?'assets/audio/music/':state.audioTab==='sfx'?'assets/audio/sfx/':'assets/audio/voice/'}</p></div><div class="file-picker">${state.audioTab==='voice'?`<select id="voice-character" class="text-input">${state.characters.map(c=>`<option value="${escapeHtml(c.id)}">${escapeHtml(c.name)}</option>`).join('')}</select>`:''}<input type="file" id="audio-upload" accept=".mp3,.wav,.ogg,.flac,.m4a"><button id="upload-audio" class="btn btn-secondary btn-small">Enviar áudio</button></div></div>
      ${entries.length?entries.map(item=>`<div class="audio-row"><div><strong>${escapeHtml(item.name)}</strong><small>${escapeHtml(item.group)} · ${escapeHtml(item.path)}</small></div><audio controls preload="none" src="${escapeHtml(item.url)}"></audio></div>`).join(''):'<div class="empty-state"><p>Nenhum áudio encontrado nesta categoria.</p></div>'}</div>`;
    document.querySelectorAll('[data-audio-tab]').forEach(b=>b.addEventListener('click',()=>{state.audioTab=b.dataset.audioTab;renderAudio();}));
    $('upload-audio').addEventListener('click',()=>uploadSelected(state.audioTab==='voice'?'voice:'+($('voice-character')?.value||'viktor-krayller'):state.audioTab,$('audio-upload')));
  }

  async function renderStories() {
    state.stories=(await api('/api/stories')).items;if(!state.storyId||!state.stories.some(s=>s.id===state.storyId))state.storyId=state.stories[0]?.id||'';
    const selected=state.stories.find(s=>s.id===state.storyId);
    $('view-root').innerHTML=`${pageHead('CAMPAIGN CONTENT','Histórias','Edite os textos de campanha em story/<id>/historia.txt.',`<span class="tag">${state.stories.length} histórias</span>`)}
      <div class="editor-layout"><section class="card list-card"><div class="list-header"><h2>Personagens / histórias</h2><p class="muted small">story/</p></div><div class="list-scroll">${state.stories.map(s=>`<button class="list-row ${s.id===state.storyId?'selected':''}" data-story="${escapeHtml(s.id)}"><div class="list-thumb initial">TXT</div><span><strong>${escapeHtml(s.id)}</strong><small>${s.exists?'historia.txt':'arquivo ainda não criado'}</small></span></button>`).join('')}</div></section>
      <div class="card card-pad">${selected?`<div class="section-title"><div><h2>${escapeHtml(selected.id)}</h2><p>story/${escapeHtml(selected.id)}/historia.txt</p></div></div><div class="field"><label>Texto da história</label><textarea id="story-content" style="min-height:430px">${escapeHtml(selected.text)}</textarea></div><div class="toolbar" style="margin-top:14px"><span class="muted small">Backup .bak criado antes da primeira alteração.</span><button class="btn btn-primary" id="save-story">Salvar história</button></div>`:'<div class="empty-state">Nenhum arquivo de história encontrado.</div>'}</div></div>`;
    document.querySelectorAll('[data-story]').forEach(b=>b.addEventListener('click',()=>{state.storyId=b.dataset.story;renderStories();}));
    $('save-story')?.addEventListener('click',async()=>{try{await api(`/api/stories/${encodeURIComponent(state.storyId)}`,jsonOptions('PUT',{text:$('story-content').value}));toast('História salva.');}catch(err){toast(err.message,'error');}});
  }

  async function renderMoves() {
    state.moves=(await api('/api/moves')).moves;
    const fields=[['startup','Startup (frames)'],['active','Active (frames)'],['recovery','Recovery (frames)'],['damage','Dano'],['reach','Alcance'],['y_offset','Altura do ataque'],['stun','Stun'],['knockback','Knockback']];
    $('view-root').innerHTML=`${pageHead('COMBAT SYSTEM','Golpes e dano','Edite os parâmetros globais de soco, chute, rasteira e especial. 60 frames equivalem a aproximadamente um segundo.',`<button class="btn btn-primary" id="save-moves">Salvar parâmetros</button>`)}
      <div class="grid grid-2">${Object.entries(state.moves).map(([id,values])=>`<div class="card card-pad"><div class="section-title"><div><h2>${{punch:'Soco',kick:'Chute',sweep:'Rasteira',special:'Especial'}[id]}</h2><p>[${id}] em data/moves.ini</p></div><span class="tag">${Number(values.startup)+Number(values.active)+Number(values.recovery)} frames</span></div><div class="form-grid">${fields.map(([key,label])=>field(`moves.${id}.${key}`,label,values[key],'number',`step="${['reach','y_offset','knockback'].includes(key)?'0.1':'1'}" min="0"`)).join('')}</div></div>`).join('')}</div>
      <div class="notice" style="margin-top:15px">Os valores desta tela são parâmetros globais do motor. Overrides específicos por personagem ficam em character.json → moves e usam as chaves abreviadas up, act, rec, dmg, reach, y, stun e kb.</div>`;
    $('save-moves').addEventListener('click',async()=>{const moves=clone(state.moves);document.querySelectorAll('[data-bind^="moves."]').forEach(el=>{const p=el.dataset.bind.split('.');moves[p[1]][p[2]]=Number(el.value);});try{await api('/api/moves',jsonOptions('PUT',{moves}));toast('Parâmetros globais de golpes salvos em data/moves.ini.');await renderMoves();}catch(err){toast(err.message,'error');}});
  }

  async function renderRules() {
    state.rules=(await api('/api/rules')).rules;const r=state.rules;
    $('view-root').innerHTML=`${pageHead('ENGINE CONFIGURATION','Regras gerais','Configurações de simulação consumidas por src/config.rs. A resolução lógica do motor continua fixa em 960 × 540.',`<button class="btn btn-primary" id="save-rules">Salvar regras</button>`)}
      <div class="grid grid-2">
        ${card(`<div class="section-title"><div><h2>Partida e rounds</h2><p>Tempo e condições de vitória.</p></div></div><div class="form-grid">${field('rules.round_seconds','Duração do round (segundos)',r.round_seconds,'number','min="1" max="999"')}${field('rules.rounds_to_win','Rounds para vencer',r.rounds_to_win,'number','min="1" max="5"')}${field('rules.cpu_difficulty','Dificuldade da CPU (0 fácil, 1 normal, 2 difícil)',r.cpu_difficulty,'number','min="0" max="2"')}${field('rules.stage_name','Nome padrão do cenário',r.stage_name)}</div>`)}
        ${card(`<div class="section-title"><div><h2>Física</h2><p>Parâmetros globais do movimento.</p></div></div><div class="form-grid">${field('rules.ground','Altura padrão do chão',r.ground,'number','min="0" max="540" step="0.1"')}${field('rules.gravity','Gravidade',r.gravity,'number','min="0.01" max="100" step="0.01"')}${field('rules.jump_velocity','Velocidade do salto',r.jump_velocity,'number','min="1" max="100" step="0.1"')}</div><div class="notice" style="margin-top:13px">Os cenários podem sobrescrever a altura do chão individualmente em stages.ini.</div>`)}
        ${card(`<div class="section-title"><div><h2>Cor do céu (fallback)</h2><p>RGB utilizado quando a imagem do cenário não está disponível.</p></div></div><div class="form-grid cols-3">${field('rules.sky_red','R',r.sky_red,'number','min="0" max="255"')}${field('rules.sky_green','G',r.sky_green,'number','min="0" max="255"')}${field('rules.sky_blue','B',r.sky_blue,'number','min="0" max="255"')}</div>`)}
        ${card(`<div class="section-title"><div><h2>Cor do piso (fallback)</h2><p>RGB do chão desenhado no modo de fallback.</p></div></div><div class="form-grid cols-3">${field('rules.floor_red','R',r.floor_red,'number','min="0" max="255"')}${field('rules.floor_green','G',r.floor_green,'number','min="0" max="255"')}${field('rules.floor_blue','B',r.floor_blue,'number','min="0" max="255"')}</div>`)}</div>
      <div class="notice warning" style="margin-top:15px">As alterações serão carregadas pelo jogo na próxima inicialização. Faça backup antes de modificar regras de física e dificuldade.</div>`;
    $('save-rules').addEventListener('click',async()=>{const rules=clone(state.rules);document.querySelectorAll('[data-bind^="rules."]').forEach(el=>{const key=el.dataset.bind.slice(6);rules[key]=el.type==='number'?Number(el.value):el.value;});try{await api('/api/rules',jsonOptions('PUT',{rules}));toast('Regras salvas em data/engine.ini.');await renderRules();}catch(err){toast(err.message,'error');}});
  }

  async function renderAssets() {
    if (!state.characters.length) state.characters=(await api('/api/characters')).items;
    const categories=[['stages','Imagens de fases'],['effects','Efeitos visuais'],['music','Músicas'],['sfx','Efeitos sonoros'],['voice','Vozes']];
    const data=await api(`/api/assets?category=${state.assetCategory}`);state.assets=data.items;
    const visual=['stages','effects'].includes(state.assetCategory);const accept=visual?'image/*':'.mp3,.wav,.ogg,.flac,.m4a';
    $('view-root').innerHTML=`${pageHead('ASSET PIPELINE','Gerenciador de recursos','Explore os diretórios reais de imagens e áudio. Arquivos removidos vão para .bfc_studio_trash, não são apagados definitivamente.')}
      <div class="tabs">${categories.map(([key,label])=>`<button class="tab ${state.assetCategory===key?'active':''}" data-asset-tab="${key}">${label}</button>`).join('')}</div>
      <div class="card card-pad"><div class="toolbar"><div><h2>${categories.find(c=>c[0]===state.assetCategory)[1]} <span class="tag">${state.assets.length}</span></h2><p class="muted small">${state.assetCategory==='stages'?'assets/images/stages/':state.assetCategory==='effects'?'assets/images/effects/':state.assetCategory==='music'?'assets/audio/music/':state.assetCategory==='sfx'?'assets/audio/sfx/':'assets/audio/voice/'}</p></div><div class="file-picker">${state.assetCategory==='voice'?`<select id="voice-character-assets" class="text-input">${state.characters.map(c=>`<option value="${escapeHtml(c.id)}">${escapeHtml(c.name)}</option>`).join('')}</select>`:''}<input id="asset-upload-file" type="file" accept="${accept}"><button class="btn btn-secondary btn-small" id="asset-upload">Adicionar arquivo</button></div></div>
      <div class="asset-grid">${state.assets.map(item=>visual?`<div class="asset-card"><img src="${escapeHtml(item.url)}"><strong>${escapeHtml(item.name)}</strong><small>${fmtBytes(item.bytes)} · ${escapeHtml(item.path)}</small><button class="btn btn-danger btn-small" style="margin-top:9px" data-delete-file="${escapeHtml(item.path)}">Mover para lixeira</button></div>`:`<div class="asset-card"><div class="metric-icon">♫</div><strong>${escapeHtml(item.name)}</strong><small>${fmtBytes(item.bytes)} · ${escapeHtml(item.path)}</small><audio controls preload="none" src="${escapeHtml(item.url)}" style="width:100%;margin-top:8px"></audio><button class="btn btn-danger btn-small" style="margin-top:9px" data-delete-file="${escapeHtml(item.path)}">Mover para lixeira</button></div>`).join('')||'<div class="empty-state">Nenhum recurso encontrado nesta categoria.</div>'}</div></div>`;
    document.querySelectorAll('[data-asset-tab]').forEach(b=>b.addEventListener('click',()=>{state.assetCategory=b.dataset.assetTab;renderAssets();}));
    $('asset-upload').addEventListener('click',()=>uploadSelected(state.assetCategory==='voice'?'voice:'+($('voice-character-assets')?.value||'viktor-krayller'):state.assetCategory,$('asset-upload-file')));
    document.querySelectorAll('[data-delete-file]').forEach(b=>b.addEventListener('click',async()=>{if(!confirm(`Mover ${b.dataset.deleteFile} para a lixeira do projeto?`))return;try{await api('/api/file',jsonOptions('DELETE',{path:b.dataset.deleteFile}));toast('Arquivo movido para .bfc_studio_trash.');renderAssets();}catch(err){toast(err.message,'error');}}));
  }

  function renderConverter() {
    $('view-root').innerHTML=`${pageHead('IMAGE PIPELINE','Conversor e normalizador PNG','Detecta o formato real pelo conteúdo, converte sprites e cenários para PNG e atualiza caminhos explícitos nos arquivos de configuração.',`<button class="btn btn-primary" id="run-converter">▶ Executar conversão</button>`)}
      <div class="grid grid-2">
        ${card(`<div class="section-title"><div><h2>Sprites de personagens</h2><p>chars/<id>/*</p></div><span class="tag good">PNG de saída</span></div><p class="muted small">Percorre os diretórios de cada personagem registrado no elenco. Imagens com extensão falsa são identificadas pelo conteúdo. JPEG, BMP, GIF, WebP e outros formatos decodificados pelo Pillow são convertidos.</p><div class="notice">Não aplica chroma key automaticamente. O motor Rust já possui configuração chroma por personagem; assim não se apagam cores legítimas das roupas.</div>`)}
        ${card(`<div class="section-title"><div><h2>Imagens de cenários</h2><p>assets/images/stages/*</p></div><span class="tag good">Sem chroma key</span></div><p class="muted small">Converte imagens das fases para PNG e ajusta os caminhos em data/stages.ini. O conteúdo verde das paisagens é preservado integralmente.</p><div class="notice warning">Se o destino PNG já existir, o conversor não o sobrescreve. Se um arquivo .png contiver outro formato, o original é preservado como .original.bak.</div>`)}
      </div>
      <div class="card card-pad" style="margin-top:16px"><div class="section-title"><div><h2>Resultado da conversão</h2><p id="converter-summary">Aguardando execução.</p></div></div><div id="converter-log" class="notice">A conversão preserva os arquivos originais quando a extensão muda. Para um arquivo com extensão .png falsa, cria uma cópia de segurança antes de substituir.</div></div>`;
    $('run-converter').addEventListener('click',async()=>{
      const btn=$('run-converter');btn.disabled=true;btn.textContent='Processando...';$('converter-summary').textContent='Analisando sprites e imagens de fases...';
      try{const res=await api('/api/tools/convert-images',{method:'POST'});$('converter-summary').textContent=`Convertidos: ${res.summary.converted} · PNGs válidos: ${res.summary.already_png} · Colisões: ${res.summary.collisions} · Falhas: ${res.summary.errors}`;let html=`<div class="notice success">${escapeHtml(res.note)}</div>`;
        const sections=[['Convertidos',res.converted],['Destinos existentes (não sobrescritos)',res.collisions],['Falhas',res.errors],['Referências atualizadas',res.references_updated]];
        for(const [title,items] of sections){html+=`<div style="margin-top:13px"><strong class="small">${title} (${items.length})</strong><div class="muted small" style="margin-top:7px;line-height:1.8">${items.length?items.map(i=>escapeHtml(typeof i==='string'?i:`${i.from} → ${i.to} (${i.format})`)).join('<br>'):'Nenhum.'}</div></div>`;}
        $('converter-log').innerHTML=html;toast('Processamento de imagens finalizado.');
      }catch(err){$('converter-log').textContent=err.message;toast(err.message,'error');}finally{btn.disabled=false;btn.textContent='▶ Executar conversão';}
    });
  }

  async function renderValidator() {
    state.validation=await api('/api/validation');const v=state.validation;const s=v.summary;
    $('view-root').innerHTML=`${pageHead('PROJECT DIAGNOSTICS','Validação do projeto','Verifique referências, estruturas JSON, arquivos de fases e integridade básica das imagens.',`<button class="btn btn-primary" id="rerun-validation">Executar novamente</button>`)}
      <div class="grid grid-3">${metric('Erros',s.errors,'Itens que precisam de atenção','!')}${metric('Avisos',s.warnings,'Referências ou recursos opcionais','△')}${metric('Personagens / fases',`${s.characters} / ${s.stages}`,'Registros encontrados','✓')}</div>
      <div class="card card-pad" style="margin-top:16px"><div class="section-title"><div><h2>Relatório de diagnóstico</h2><p>Os avisos não impedem necessariamente o jogo de iniciar.</p></div><span class="tag ${s.errors?'bad':s.warnings?'warn':'good'}">${s.errors?'Ação necessária':s.warnings?'Revisar avisos':'Estrutura consistente'}</span></div>
      ${v.issues.length?v.issues.map(item=>`<div class="issue"><span class="issue-icon">${item.level==='error'?'⛔':'⚠'}</span><div><strong>${escapeHtml(item.path)}</strong><small>${escapeHtml(item.message)}</small></div><span class="tag ${item.level==='error'?'bad':'warn'}">${item.level==='error'?'Erro':'Aviso'}</span></div>`).join(''):'<div class="notice success">Nenhum problema encontrado nos arquivos verificados.</div>'}</div>`;
    $('rerun-validation').addEventListener('click',()=>renderValidator());
  }

  document.querySelectorAll('.nav-item[data-view]').forEach(button=>button.addEventListener('click',()=>loadView(button.dataset.view)));
  $('open-project-btn').addEventListener('click',()=>{
    const path=prompt('Informe o caminho completo da pasta raiz do Brazil Fight Combat:',state.project||'');
    if(path===null||!path.trim())return;
    openProject(path.trim()).catch(err=>toast(err.message,'error'));
  });
  $('main-nav').addEventListener('click',e=>{const b=e.target.closest('[data-go]');if(b)loadView(b.dataset.go);});
  initialize();
})();
