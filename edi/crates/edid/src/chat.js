// EDI · cara viva y conversación (compartido por el widget y el dashboard; sin dependencias).
// Cara: SVG vectorial en línea → parpadea y mueve la boca mientras habla.
// Conversación: tus frases y las suyas, con 👍 👎 ✏️ en cada respuesta y el porqué (candidatas y valencia).
// Entrada: texto (Enter), voz (🎤, reconocimiento del navegador) y atajos:
//   «+» o «👍» premia su última respuesta · «-» o «👎» la castiga · «= texto» le enseña qué debía decir.
// Órdenes «/…» (diario, notas, recordatorios…): las contesta el sistema, no su cerebro; Tab completa.
(() => {
  const J = (u, o) => fetch(u, o).then(r => r.json());
  const store = {
    get: (k, d) => { try { const v = localStorage.getItem('edi.' + k); return v === null ? d : JSON.parse(v) } catch (e) { return d } },
    set: (k, v) => { try { localStorage.setItem('edi.' + k, JSON.stringify(v)) } catch (e) { } },
  };
  const esc = s => String(s).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
  const CMDS = [['/ayuda', 'todas las órdenes'], ['/diario', '[horas] lo que percibí'], ['/nota', 'TEXTO apunto una nota'], ['/notas', '[palabra] tus notas'],
    ['/recuerdame', '10m|2h|18:30 TEXTO aviso'], ['/recordatorios', 'pendientes'], ['/borra', 'N quita un recordatorio'], ['/recuerda', 'PALABRA qué asocio'],
    ['/enseña', 'PREGUNTA => RESPUESTA'], ['/aprendido', 'lo que llevo aprendido'], ['/estado', 'cómo estoy'], ['/duerme', 'a dormir']];
  const SIS = { ayuda: 'ayuda', diario: 'diario', nota: 'notas', recordatorio: 'recordatorio', aviso: '⏰ aviso', alerta: '⚠ alerta', recuerdo: 'recuerdo', enseñanza: 'enseñanza', estado: 'estado' };
  const css = `
.edi-face svg{width:100%;height:auto;display:block;overflow:visible}
.edi-face #ojo-i,.edi-face #ojo-d{transition:transform .07s ease-in}
.edi-face.blink #ojo-i,.edi-face.blink #ojo-d{transform:scaleY(.08)}
.edi-face.talking #boca{animation:edi-talk .18s ease-in-out infinite alternate}
@keyframes edi-talk{from{transform:scale(1,1)}to{transform:scale(1.06,.5)}}
.edi-log{display:flex;flex-direction:column;gap:5px;overflow-y:auto;overscroll-behavior:contain;scrollbar-width:thin}
.edi-t{max-width:88%;padding:4px 8px;border-radius:8px;white-space:pre-wrap;word-break:break-word;line-height:1.35}
.edi-t.tu{align-self:flex-end;background:#16324a;color:#cfe6f5;border-bottom-right-radius:2px}
.edi-t.edi{align-self:flex-start;background:rgba(10,24,30,.9);border:1px solid #3c8f8f;color:#e8fbff;border-bottom-left-radius:2px}
.edi-t.fb{align-self:flex-end;background:none;color:#7f98a6;padding:0 4px;font-size:.92em}
.edi-t.ens{align-self:flex-end;background:#2c2448;color:#e2d6ff}
.edi-t.ord{align-self:flex-end;background:none;border:1px dashed #2a4a66;color:#9fc4cc}
.edi-t.sis{align-self:stretch;max-width:100%;background:#0a1420;border-left:2px solid #6b8fa6;color:#c4d8e2;border-radius:3px;font-size:.95em}
.edi-t.sis.alerta{border-left-color:#f6ad55;background:#20160a}.edi-t.sis.aviso{border-left-color:#4fd1c5;background:#0a1f1d;color:#e8fbff;font-size:1.05em}
.edi-t.sis .k{margin:0 0 2px}
@keyframes edi-pulse{0%,100%{box-shadow:none}50%{box-shadow:0 0 0 2px #4fd1c5,0 0 24px #4fd1c588}}
.edi-alert{animation:edi-pulse 1s ease-in-out 3}
.edi-hint{font-size:.85em;color:#9fc4cc}.edi-hint b{color:#4fd1c5;font-weight:500}
.edi-chips{display:flex;flex-wrap:wrap;gap:4px;margin-top:5px}
.edi-chips button{all:unset;cursor:pointer;font-size:.85em;padding:1px 7px;border:1px solid #1f3a4f;border-radius:10px;color:#9fc4cc}
.edi-chips button:hover{border-color:#4fd1c5;color:#e8fbff}
.edi-t .k{display:block;font-size:.8em;color:#7f98a6;margin-top:2px}
.edi-t .a{display:flex;gap:2px;margin-top:3px}
.edi-t .a button{all:unset;cursor:pointer;padding:0 5px;border-radius:4px;opacity:.55;font-size:.95em}
.edi-t .a button:hover{opacity:1;background:#ffffff14}.edi-t .a button.on{opacity:1;background:#4fd1c533}
.edi-t .why{display:none;font-size:.82em;color:#9fc4cc;margin-top:3px}.edi-t.showwhy .why{display:block}
.edi-t input{width:100%;box-sizing:border-box;margin-top:4px;background:#050b16;color:#e8fbff;border:1px solid #6b5bd6;border-radius:4px;padding:3px 6px;font:inherit}
.edi-think{min-height:1.2em;font-size:.85em;color:#7fd8d8;opacity:.85}
.edi-in{display:flex;gap:4px;align-items:center}
.edi-in input{flex:1;min-width:0;background:#050b16;color:#d6ecf3;border:1px solid #2a4a66;border-radius:6px;padding:6px 9px;font:inherit}
.edi-in input:focus{outline:none;border-color:#4fd1c5}
.edi-in button{all:unset;cursor:pointer;width:26px;height:26px;display:grid;place-items:center;border-radius:6px;background:#0d1d2a;border:1px solid #2a4a66;font-size:13px}
.edi-in button:hover{border-color:#4fd1c5}.edi-in button.rec{background:#5a1630;border-color:#ff6b8b}.edi-in button[disabled]{opacity:.35;cursor:default}`;
  if (!document.getElementById('edi-css')) { const s = document.createElement('style'); s.id = 'edi-css'; s.textContent = css; document.head.appendChild(s) }

  // ---------- cara ----------
  class Face {
    constructor(el) {
      this.el = el; this.id = 0; this.cache = {}; el.classList.add('edi-face');
      const blink = () => { this.el.classList.add('blink'); setTimeout(() => this.el.classList.remove('blink'), 140); setTimeout(blink, 2200 + Math.random() * 4200) };
      setTimeout(blink, 1500);
    }
    async set(f) {
      if (!f || f.id === this.id) return; this.id = f.id;
      const u = `/face/${String(f.id).padStart(2, '0')}_${f.name}.svg`;
      try { if (!this.cache[u]) { const r = await fetch(u); if (!r.ok) throw r.status; this.cache[u] = await r.text() } if (this.id === f.id) this.el.innerHTML = this.cache[u] } catch (e) { this.id = 0 }
    }
    talk(on) { this.el.classList.toggle('talking', !!on) }
  }

  // ---------- voz ----------
  let voice = null;
  const pickVoice = () => { const vs = speechSynthesis.getVoices(); voice = vs.find(v => /^es(-|_)ES/i.test(v.lang)) || vs.find(v => /^es/i.test(v.lang)) || null };
  if ('speechSynthesis' in window) { pickVoice(); speechSynthesis.onvoiceschanged = pickVoice }

  // ---------- conversación ----------
  function mount(root, opt = {}) {
    const q = s => root.querySelector(`[data-edi=${s}]`);
    const face = q('face') ? new Face(q('face')) : null, log = q('log'), inp = q('input');
    const max = opt.max || 60, turns = new Map();
    let last = 0, first = true, tts = store.get('tts', false);
    log.classList.add('edi-log');
    // pegado al final mientras no subas a leer; también si el panel cambia de tamaño
    let stick = true;
    const bottom = () => { log.scrollTop = log.scrollHeight };
    log.addEventListener('scroll', () => { stick = log.scrollHeight - log.scrollTop - log.clientHeight < 40 });
    if ('ResizeObserver' in window) new ResizeObserver(() => { if (stick) bottom() }).observe(log);
    inp.classList.add('edi-in');
    inp.innerHTML = `<button data-k=mic title="Hablar (voz)">🎤</button><input placeholder="${opt.placeholder || 'Háblale… (Enter · / órdenes)'}" autocomplete=off title="Enter: enviar · «+»/«-»: premiar o castigar su última respuesta · «= texto»: enseñarle qué decir · «/»: órdenes (/ayuda)"><button data-k=send title="Enviar">➤</button><button data-k=tts title="Que hable en voz alta"></button>`;
    const box = inp.querySelector('input'), think = document.createElement('div');
    if (opt.chips) {
      // accesos rápidos a sus usos
      const ch = document.createElement('div'); ch.className = 'edi-chips';
      ch.innerHTML = [['📔 diario', '/diario'], ['📝 notas', '/notas'], ['⏰ recordatorios', '/recordatorios'], ['🧠 aprendido', '/aprendido'], ['❓ ayuda', '/ayuda']]
        .map(([l, c]) => `<button data-c="${c}">${l}</button>`).join('');
      ch.onclick = e => { const c = e.target.closest('button'); if (c) send(c.dataset.c) };
      inp.after(ch);
    }
    think.className = 'edi-think'; log.after(think);
    const ttsBtn = inp.querySelector('[data-k=tts]');
    const paintTts = () => ttsBtn.textContent = tts ? '🔊' : '🔇';
    paintTts();
    ttsBtn.onclick = () => { tts = !tts; store.set('tts', tts); paintTts(); if (!tts && 'speechSynthesis' in window) speechSynthesis.cancel() };
    const say = text => {
      if (!tts || !('speechSynthesis' in window)) return;
      const u = new SpeechSynthesisUtterance(text); u.lang = 'es-ES'; if (voice) u.voice = voice; u.rate = 1.02; u.pitch = 1.1;
      u.onstart = () => face && face.talk(true); u.onend = u.onerror = () => face && face.talk(false);
      speechSynthesis.speak(u);
    };
    const lastEdi = () => [...turns.values()].reverse().find(t => t.who === 'edi');
    const post = (u, body) => fetch(u, { method: 'POST', body });
    const reward = (id, v) => post(`/reward?turn=${id || ''}&v=${v > 0 ? '+' : '-'}`, v > 0 ? '+' : '-');
    const teach = (id, text) => post(`/teach?turn=${id || ''}`, text);
    let hist = store.get('hist', []), hi = -1;
    async function send(raw) {
      const t = raw.trim(); if (!t) return;
      hist = [t, ...hist.filter(x => x !== t)].slice(0, 30); store.set('hist', hist); hi = -1;
      const le = lastEdi();
      if (/^(\+|👍)$/.test(t)) return reward(le && le.id, 1);
      if (/^(-|👎)$/.test(t)) return reward(le && le.id, -1);
      if (t.startsWith('=')) return teach(le && le.id, t.slice(1).trim());
      await post('/say?wait=0', t);
      hint();
      poll();
    }
    // ayuda de órdenes mientras escribes «/…»; Tab completa la primera
    const fold = x => x.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase();
    const matches = () => {
      const v = box.value.trimStart(), w = fold(v.split(/\s/)[0]);
      return w.startsWith('/') ? CMDS.filter(([c]) => /\s/.test(v) ? fold(c) === w : fold(c).startsWith(w)) : [];
    };
    function hint() {
      const m = matches();
      if (!m.length) { if (think.classList.contains('edi-hint')) { think.classList.remove('edi-hint'); think.textContent = '' } return }
      think.classList.add('edi-hint'); think.innerHTML = m.slice(0, 4).map(([c, d]) => `<b>${esc(c)}</b> ${esc(d)}`).join(' · ');
    }
    box.oninput = hint;
    box.onkeydown = e => {
      if (e.key === 'Tab' && matches().length && !/\s/.test(box.value.trim())) { e.preventDefault(); box.value = matches()[0][0] + ' '; hint() }
      else if (e.key === 'Enter') { e.preventDefault(); const v = box.value; box.value = ''; send(v) }
      else if (e.key === 'ArrowUp' && hist.length) { e.preventDefault(); hi = Math.min(hi + 1, hist.length - 1); box.value = hist[hi] }
      else if (e.key === 'ArrowDown') { e.preventDefault(); hi = Math.max(hi - 1, -1); box.value = hi < 0 ? '' : hist[hi] }
      else if (e.key === 'Escape') box.value = '';
    };
    inp.querySelector('[data-k=send]').onclick = () => { const v = box.value; box.value = ''; send(v); box.focus() };
    // reconocimiento de voz del navegador (Edge/Chrome; puede usar un servicio en la nube del navegador)
    const SR = window.SpeechRecognition || window.webkitSpeechRecognition, mic = inp.querySelector('[data-k=mic]');
    if (!SR) { mic.disabled = true; mic.title = 'Este navegador no tiene reconocimiento de voz (prueba Edge o Chrome)' }
    else {
      let rec = null;
      mic.onclick = () => {
        if (rec) { rec.stop(); return }
        rec = new SR(); rec.lang = 'es-ES'; rec.interimResults = true; rec.maxAlternatives = 1;
        mic.classList.add('rec'); think.textContent = 'escuchando…';
        rec.onresult = e => { const r = e.results[e.results.length - 1]; box.value = r[0].transcript; if (r.isFinal) { const v = box.value; box.value = ''; send(v) } };
        rec.onerror = e => { think.textContent = e.error === 'network' ? 'la voz necesita la conexión del navegador' : e.error === 'not-allowed' ? 'micrófono no permitido' : 'voz: ' + e.error };
        rec.onend = () => { mic.classList.remove('rec'); rec = null; if (think.textContent === 'escuchando…') think.textContent = '' };
        rec.start();
      };
    }
    function render(t) {
      let el = log.querySelector(`[data-id="${t.id}"]`);
      const cls = t.who === 'edi' ? 'edi' : t.who === 'sistema' ? 'sis ' + t.kind : t.kind === 'feedback' ? 'fb' : t.kind === 'enseñanza' ? 'ens' : t.kind === 'orden' ? 'ord' : 'tu';
      if (!el) { el = document.createElement('div'); el.dataset.id = t.id; log.appendChild(el) }
      el.className = 'edi-t ' + cls + (el.classList.contains('showwhy') ? ' showwhy' : '');
      if (t.who === 'sistema') { el.innerHTML = `<span class=k>${esc(SIS[t.kind] || t.kind)} · sistema</span>${esc(t.text)}`; return el }
      if (t.who !== 'edi') { el.innerHTML = esc(t.kind === 'enseñanza' ? '✏️ ' + t.text : t.text); return el }
      const why = (t.cands || []).map(([w, s]) => `${esc(w)} <b>${s >= 0 ? '+' : ''}${s.toFixed(3)}</b>`).join(' · ');
      const kind = (t.kind === 'ecolalia' ? 'repite (aún no sabe qué contestar)' : t.kind === 'asociación' ? 'contesta como a algo parecido que oyó' : 'imita lo que oyó') + ((t.cands || []).length > 1 ? '; eligió su cuerpo fungiforme' : '');
      el.innerHTML = `${esc(t.text)}<span class=k>${kind}</span><div class=a><button data-a=up title="Bien dicho (dopamina de premio)" class="${t.reward > 0 ? 'on' : ''}">👍</button><button data-a=down title="Mal (dopamina de castigo)" class="${t.reward < 0 ? 'on' : ''}">👎</button><button data-a=teach title="Enséñale qué debía decir">✏️</button>${why ? '<button data-a=why title="Por qué lo dijo">?</button>' : ''}</div><div class=why>olió: ${why}</div>`;
      el.querySelector('[data-a=up]').onclick = () => { reward(t.id, 1); t.reward = 1; render(t) };
      el.querySelector('[data-a=down]').onclick = () => { reward(t.id, -1); t.reward = -1; render(t) };
      const w = el.querySelector('[data-a=why]'); if (w) w.onclick = () => el.classList.toggle('showwhy');
      el.querySelector('[data-a=teach]').onclick = () => {
        if (el.querySelector('input')) return;
        const i = document.createElement('input'); i.placeholder = '¿qué debería haber dicho? (Enter)'; el.appendChild(i); i.focus();
        i.onkeydown = e => { if (e.key === 'Enter' && i.value.trim()) { teach(t.id, i.value.trim()); i.remove() } else if (e.key === 'Escape') i.remove() };
      };
      return el;
    }
    async function poll() {
      try {
        const c = await J('/chat?since=' + last);
        let spoke = null;
        for (const t of c.turns) { turns.set(t.id, t); render(t); last = Math.max(last, t.id); if (t.who === 'edi' && !first) spoke = t }
        for (const [id, rw, kind] of c.recent || []) { const t = turns.get(id); if (t && (t.reward !== rw || t.kind !== kind)) { t.reward = rw; t.kind = kind; render(t) } }
        while (log.children.length > max) { turns.delete(+log.firstChild.dataset.id); log.firstChild.remove() }
        if (c.turns.length && (stick || c.turns.some(t => t.who === 'tú'))) { stick = true; bottom() }
        if (think.textContent !== 'escuchando…' && !think.classList.contains('edi-hint')) think.textContent = c.thinking ? `pensando… huele ${c.thinking.olidas}/${c.thinking.total} respuestas posibles` : '';
        if (spoke) say(spoke.text);
        const al = c.turns.filter(t => t.who === 'sistema' && (t.kind === 'aviso' || t.kind === 'alerta'));
        if (al.length && !first) {
          // un aviso o una alerta: que se note (y que lo diga si tiene voz)
          const a = al[al.length - 1], el = opt.alertEl || root;
          el.classList.remove('edi-alert'); void el.offsetWidth; el.classList.add('edi-alert');
          say(a.text.replace(/^[^\p{L}\p{N}]+/u, ''));
          if (opt.onAlert) opt.onAlert(a);
        }
        first = false;
      } catch (e) { }
    }
    poll(); setInterval(poll, 600);
    return { face, focus: () => box.focus() };
  }
  window.EDI = { mount, Face };
})();
