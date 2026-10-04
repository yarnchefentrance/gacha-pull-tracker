// Wishboard UI. Talks to the Rust backend through Tauri; falls back to built-in mock data in a plain browser.
const tauri = window.__TAURI__;
const invoke = (cmd, args) => tauri.core.invoke(cmd, args);
const $ = (s, el = document) => el.querySelector(s);
const pct = (v, d = 1) => (v * 100).toFixed(d) + '%';
const num = v => Math.round(v).toLocaleString('en-US');
const MON = 'Jan Feb Mar Apr May Jun Jul Aug Sep Oct Nov Dec'.split(' ');
const fdate = t => `${MON[+t.slice(5, 7) - 1]} ${+t.slice(8, 10)}`;
const ftime = t => `${fdate(t)}, ${t.slice(11, 16)}`;

const I = {
  overview: '<rect x="2" y="2" width="5" height="5" rx="1"/><rect x="9" y="2" width="5" height="5" rx="1"/><rect x="2" y="9" width="5" height="5" rx="1"/><rect x="9" y="9" width="5" height="5" rx="1"/>',
  history: '<path d="M2.5 4h11M2.5 8h11M2.5 12h7"/>',
  odds: '<path d="M2 13.5h12"/><path d="M2.5 11.5C6 11.5 6.5 3.5 10 3.5c2 0 2.5 1 3.5 1"/>',
  import: '<path d="M8 2v8M4.75 6.75 8 10l3.25-3.25"/><path d="M2.5 11v1.5a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1V11"/>',
  export: '<path d="M8 10V2M4.75 5.25 8 2l3.25 3.25"/><path d="M2.5 11v1.5a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1V11"/>',
  search: '<circle cx="7" cy="7" r="4.25"/><path d="m10.25 10.25 3.25 3.25"/>',
  check: '<path d="m3.5 8.5 3 3 6-7"/>',
  refresh: '<path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9"/><path d="M13.5 2.5v3h-3"/>',
  file: '<path d="M4 1.75h5l3 3v9.5H4z"/><path d="M9 1.75v3h3"/>',
};
const ico = n => `<svg class="ico" viewBox="0 0 16 16">${I[n]}</svg>`;

// ---------- Game metadata ----------
const R = (hard, soft, base, step, q, carry, rad) =>
  ({ hard_pity: hard, soft_start: soft, base_rate: base, soft_step: step, four_star_pity: 10, featured_rate: q, carries_guarantee: !!carry, capturing_radiance: !!rad });
const GAMES = [
  { id: 'genshin', title: 'Genshin Impact', short: 'Genshin', word: 'wishes', cur: 'Primogems', item: 'Intertwined Fates', pass: 'Welkin Moon', end: 'Abyss and Theater', color: 'var(--g-genshin)', uid: '712408113', region: 'Europe',
    log: 'C:\\Users\\Player\\AppData\\LocalLow\\miHoYo\\Genshin Impact\\output_log.txt', dir: 'D:\\Games\\Genshin Impact game\\GenshinImpact_Data', host: 'public-operation-hk4e-sg.hoyoverse.com/gacha_info/api/getGachaLog',
    banners: [['character', 'Character Event Wish', 'Character Event', R(90, 74, .006, .06, .5, 1, 1), 10, 67, 11], ['weapon', 'Weapon Event Wish', 'Weapon Event', R(80, 63, .007, .07, .75, 1), 2, 31, 12],
      ['standard', 'Wanderlust Invocation', 'Standard', R(90, 74, .006, .06, 1, 0), 3, 48, 13], ['chronicled', 'Chronicled Wish', 'Chronicled', R(90, 74, .006, .06, .5, 1), 1, 12, 14]],
    fc: 'Neuvillette|Furina|Arlecchino|Mavuika|Citlali|Varesa|Escoffier|Ineffa|Lauma|Flins', fw: 'Splendor of Tranquil Waters|Crimson Moon\'s Semblance|A Thousand Blazing Suns',
    std: 'Diluc|Jean|Keqing|Mona|Qiqi|Tighnari|Dehya', c4: 'Bennett|Xiangling|Xingqiu|Fischl|Sucrose|Chevreuse|Kachina|Ororon', w4: 'Favonius Sword|Sacrificial Bow|The Widsith|Rust|Favonius Lance',
    w3: 'Cool Steel|Harbinger of Dawn|Slingshot|Debate Club|Black Tassel|Raven Bow|Magic Guide', words: ['Character', 'Weapon'] },
  { id: 'hsr', title: 'Honkai: Star Rail', short: 'Star Rail', word: 'warps', cur: 'Stellar Jade', item: 'Star Rail Special Passes', pass: 'Express Supply Pass', end: 'MoC, Pure Fiction, Apocalyptic Shadow', color: 'var(--g-hsr)', uid: '704918822', region: 'Europe',
    log: 'C:\\Users\\Player\\AppData\\LocalLow\\Cognosphere\\Star Rail\\Player.log', dir: 'D:\\HoYoPlay\\games\\Star Rail Games\\StarRail_Data', host: 'public-operation-hkrpg-sg.hoyoverse.com/common/gacha_record/api/getGachaLog',
    banners: [['character', 'Character Event Warp', 'Character Event', R(90, 74, .006, .06, .5, 1), 7, 54, 21], ['weapon', 'Light Cone Event Warp', 'Light Cone', R(80, 66, .008, .07, .75, 1), 3, 22, 22], ['standard', 'Stellar Warp', 'Standard', R(90, 74, .006, .06, 1, 0), 2, 71, 23]],
    fc: 'Acheron|Firefly|Feixiao|The Herta|Aglaea|Castorice|Phainon', fw: 'Along the Passing Shore|Whereabouts Should Dreams Rest|I Venture Forth to Hunt', std: 'Himeko|Welt|Bronya|Gepard|Clara|Yanqing|Bailu',
    c4: 'Pela|Tingyun|Asta|Sushang|Hanya|Gallagher|Lynx', w4: 'Dance! Dance! Dance!|Memories of the Past|Planetary Rendezvous', w3: 'Arrows|Cornucopia|Collapsing Sky|Amber|Void|Chorus|Mutual Demise', words: ['Character', 'Light Cone'] },
  { id: 'zzz', title: 'Zenless Zone Zero', short: 'ZZZ', word: 'signals', cur: 'Polychrome', item: 'Encrypted Master Tapes', pass: 'Inter-Knot Membership', end: 'Shiyu Defense and Deadly Assault', color: 'var(--g-zzz)', uid: '1500390471', region: 'Europe',
    log: 'C:\\Users\\Player\\AppData\\LocalLow\\miHoYo\\ZenlessZoneZero\\Player.log', dir: 'D:\\HoYoPlay\\games\\ZenlessZoneZero Game\\ZenlessZoneZero_Data', host: 'public-operation-nap-sg.hoyoverse.com/common/gacha_record/api/getGachaLog',
    banners: [['character', 'Exclusive Channel', 'Exclusive', R(90, 74, .006, .06, .5, 1), 6, 38, 31], ['weapon', 'W-Engine Channel', 'W-Engine', R(80, 65, .01, .06, .75, 1), 2, 60, 32],
      ['standard', 'Stable Channel', 'Standard', R(90, 74, .006, .06, 1, 0), 2, 15, 33], ['bangboo', 'Bangboo Channel', 'Bangboo', R(80, 65, .01, .06, 1, 0), 1, 9, 34]],
    fc: 'Ellen|Zhu Yuan|Jane|Miyabi|Evelyn|Astra Yao|Yixuan', fw: 'Deep Sea Visitor|Hailstorm Shrine|Flamemaker Shaker', std: 'Grace|Rina|Koleda|Nekomata|Soldier 11|Lycaon',
    c4: 'Nicole|Anby|Soukaku|Lucy|Ben|Pulchra', w4: 'Steam Oven|Bashful Demon|Starlight Engine|Slice of Time', w3: '[Lunar] Pleniluna|[Reverb] Mark I|[Vortex] Arrow|[Cannon] Rotor', words: ['Agents', 'W-Engines'] },
  { id: 'wuwa', title: 'Wuthering Waves', short: 'WuWa', word: 'convenes', cur: 'Astrite', item: 'Radiant Tides', pass: 'Lunite Subscription', end: 'Tower of Adversity and Whimpering Wastes', color: 'var(--g-wuwa)', uid: '500127764', region: 'Europe',
    log: 'D:\\Wuthering Waves\\Wuthering Waves Game\\Client\\Saved\\Logs\\Client.log', dir: '', host: 'aki-gm-resources-oversea.aki-game.net/aki/gacha/index.html#/record',
    banners: [['character', 'Featured Resonator Convene', 'Featured Resonator', R(80, 66, .008, .07, .5, 1), 5, 44, 41], ['weapon', 'Featured Weapon Convene', 'Featured Weapon', R(80, 66, .008, .07, 1, 0), 2, 18, 42], ['standard', 'Standard Resonator Convene', 'Standard', R(80, 66, .008, .07, 1, 0), 1, 36, 43]],
    fc: 'Jiyan|Yinlin|Camellya|Carlotta|Zani|Cartethyia|Phrolova', fw: 'Verdant Summit|Stringmaster|Red Spring', std: 'Verina|Encore|Calcharo|Lingyang|Jianxin',
    c4: 'Sanhua|Baizhi|Chixia|Danjin|Mortefi|Yuanwu|Taoqi', w4: 'Lumingloss|Commando of Conviction|Hollow Mirage', w3: 'Originite: Type I|Guardian Broadblade|Guardian Sword|Guardian Pistols|Guardian Rectifier', words: ['Resonator', 'Weapon'] },
];
const G = id => GAMES.find(g => g.id === id) || GAMES[0];
const rulesOf = (g, b) => G(g).banners.find(x => x[0] === b)[3];

// ---------- Odds engine (mirror of odds.rs) ----------
const rateAt = (r, k) => k >= r.hard_pity ? 1 : k < r.soft_start ? r.base_rate : Math.min(1, r.base_rate + r.soft_step * (k - r.soft_start + 1));
function pmfFrom(r, p) {
  p = Math.min(p, r.hard_pity - 1);
  const o = []; let s = 1;
  for (let k = p + 1; k <= r.hard_pity; k++) { const q = rateAt(r, k); o.push(s * q); s *= 1 - q; }
  return o;
}
const cum = f => { let s = 0; return f.map(v => (s += v)); };
const at = (c, n) => n <= 0 ? 0 : Math.min(1, c[Math.min(n, c.length) - 1]);
function pFeatured(r, p, g, n) {
  const f = pmfFrom(r, p), F = cum(f), first = at(F, n);
  if (g || r.featured_rate >= 1) return first;
  const C = cum(pmfFrom(r, 0)); let s = 0;
  for (let j = 0; j < Math.min(n, f.length); j++) s += f[j] * at(C, n - j - 1);
  const q = r.featured_rate;
  return r.carries_guarantee ? Math.min(1, q * first + (1 - q) * s) : q * first + (1 - q) * q * s;
}
const pAny = (r, p, n) => at(cum(pmfFrom(r, p)), n);
const expected = r => pmfFrom(r, 0).reduce((a, v, i) => a + v * (i + 1), 0);
const forConf = (r, p, g, t) => { for (let n = 1; n <= r.hard_pity * 2; n++) if (pFeatured(r, p, g, n) >= t) return n; return r.hard_pity * 2; };
function rng(seed) { let a = seed >>> 0; return () => { a = (a + 0x6D2B79F5) >>> 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
function luck(r, observed, count, trials, seed) {
  const rnd = rng(seed), bin_start = Math.floor(r.hard_pity / 3), bins = new Array(Math.floor((r.hard_pity - bin_start) / 2) + 1).fill(0);
  let worse = 0;
  for (let t = 0; t < trials; t++) {
    let s = 0;
    for (let i = 0; i < count; i++) { let k = 1; while (k < r.hard_pity && rnd() >= rateAt(r, k)) k++; s += k; }
    const m = s / count; if (m > observed) worse++;
    bins[Math.min(bins.length - 1, Math.floor((Math.max(m, bin_start) - bin_start) / 2))]++;
  }
  return { observed, expected: expected(r), percentile: worse * 100 / trials, samples: trials, bin_start, bin_width: 2, bins };
}

// ---------- Mock backend (mirror of sample.rs / pity.rs / planner.rs) ----------
function genHistory(g) {
  const all = [];
  g.banners.forEach(([bid, , , r, fives, fin, seed], bi) => {
    const rnd = rng(seed * 7919), pick = l => { const a = l.split('|'); return a[Math.floor(rnd() * a.length)]; };
    const feat = (bid === 'weapon' ? g.fw : g.fc).split('|'), out = [];
    let pity = 0, p4 = 0, got = 0, gua = false, fi = 0;
    while (!(got === fives && pity === fin)) {
      pity++; p4++;
      if (got < fives && rnd() < rateAt(r, pity)) {
        const win = gua || r.featured_rate >= 1 || rnd() < r.featured_rate;
        let name, featured = null;
        if (bid === 'bangboo') name = pick('Butler|Sharkboo|Plugboo|Amillion');
        else if (bid === 'standard') name = pick(g.std);
        else if (win) { fi = (fi + 1 + Math.floor(rnd() * 2)) % feat.length; name = feat[fi]; featured = true; }
        else { name = pick(g.std); featured = false; }
        gua = !win && r.carries_guarantee;
        out.push({ name, rarity: 5, featured, item_type: bid === 'weapon' ? g.words[1] : bid === 'bangboo' ? 'Bangboo' : g.words[0] });
        got++; pity = 0; p4 = 0;
      } else if (p4 >= 10 || rnd() < .051) {
        const c = bid !== 'weapon' && rnd() < .66;
        out.push({ name: pick(c ? g.c4 : g.w4), rarity: 4, item_type: g.words[c ? 0 : 1] }); p4 = 0;
      } else out.push({ name: pick(g.w3), rarity: 3, item_type: g.words[1] });
    }
    let t = 1794087600 - bi * 5400;
    for (let i = out.length - 1, k = 0; i >= 0; i--, k++) {
      if (k && k % 10 === 0) t -= 14400 + Math.floor(rnd() * 200000);
      Object.assign(out[i], { banner: bid, id: `${t}${bi}${String(i).padStart(4, '0')}`, time: new Date(t * 1000).toISOString().slice(0, 19).replace('T', ' ') });
    }
    all.push(...out);
  });
  return all.sort((a, b) => a.time < b.time ? -1 : a.time > b.time ? 1 : a.id < b.id ? -1 : 1);
}
function analyze(g, b, pulls) {
  const r = rulesOf(g.id, b[0]), std = g.std.split('|'), ff = r.featured_rate < 1;
  const st = { banner: b[0], label: b[1], rules: r, total: 0, five_stars: 0, four_stars: 0, pity: 0, pity4: 0, guaranteed: false, wins: 0, losses: 0, lost_streak: 0, hits: [] };
  let sum = 0;
  for (const p of pulls) {
    if (p.banner !== b[0]) continue;
    st.total++; st.pity++; st.pity4++;
    if (p.rarity === 5) {
      const feat = p.featured ?? !std.includes(p.name);
      const o = !ff ? 'standard' : st.guaranteed ? 'guaranteed' : feat ? 'won' : 'lost';
      if (o === 'won') { st.wins++; st.lost_streak = 0; } if (o === 'lost') { st.losses++; st.lost_streak++; }
      st.guaranteed = o === 'lost' && r.carries_guarantee;
      st.five_stars++; sum += st.pity; st.hits.push({ name: p.name, time: p.time, pity: st.pity, outcome: o }); st.pity = 0; st.pity4 = 0;
    } else if (p.rarity === 4) { st.four_stars++; st.pity4 = 0; }
  }
  st.avg_pity = st.five_stars ? sum / st.five_stars : 0;
  st.in_soft_pity = st.pity + 1 >= r.soft_start;
  st.win_rate = st.wins + st.losses ? st.wins / (st.wins + st.losses) : null;
  st.notes = [st.in_soft_pity ? `Soft pity active: rate is climbing, hard pity in ${r.hard_pity - st.pity} pulls.` : `${r.soft_start - st.pity} pulls to soft pity (pull ${r.soft_start}), ${r.hard_pity - st.pity} to hard pity.`];
  if (st.guaranteed) st.notes.push('Next 5★ is guaranteed to be the featured item.');
  else if (ff) st.notes.push(`Next 5★ is a ${r.featured_rate * 100}/${100 - r.featured_rate * 100}.`);
  if (r.capturing_radiance) st.notes.push(`Capturing Radiance: ${st.lost_streak} consecutive lost 50/50 on record. Losing streaks raise the next win chance.`);
  return st;
}
const INCOME = { genshin: [60, 90, 1600, 30], hsr: [60, 90, 2400, 42], zzz: [60, 90, 1560, 28], wuwa: [60, 90, 1600, 28] };
function plan(i) {
  const [daily, pass, end, cyc] = INCOME[i.game], d = i.days, g = G(i.game), lines = [[`${g.cur} on hand`, i.currency]];
  if (i.pulls_owned) lines.push([g.item, i.pulls_owned * 160]);
  if (i.daily_missions) lines.push([`Daily missions, ${d} days`, daily * d]);
  if (i.monthly_pass) lines.push([g.pass, pass * d]);
  const c = Math.floor(d / cyc); if (c && i.endgame_clear) lines.push([`Endgame, ${c} reset${c > 1 ? 's' : ''}`, Math.round(end * i.endgame_clear * c)]);
  if (i.events_per_patch) lines.push(['Events and patch rewards', Math.round(i.events_per_patch * d / 42)]);
  if (i.battle_pass) lines.push(['Battle pass pulls', Math.floor(4 * d / 42) * 160]);
  const total = lines.reduce((a, l) => a + l[1], 0), pulls = Math.floor(total / 160), r = rulesOf(i.game, i.banner);
  return { currency_total: total, pulls, lines: lines.map(([label, amount]) => ({ label, amount })), p_any: pAny(r, i.pity, pulls), p_featured: pFeatured(r, i.pity, i.guaranteed, pulls), shortfall_90: Math.max(0, forConf(r, i.pity, i.guaranteed, .9) - pulls) };
}
const mock = {
  cache: {},
  pulls(id) { return this.cache[id] || (this.cache[id] = genHistory(G(id))); },
  overview(id) {
    const g = G(id), pulls = this.pulls(id), banners = g.banners.map(b => analyze(g, b, pulls)), c = banners[0];
    return { game: id, title: g.title, uid: g.uid, is_sample: !location.search.includes('demo'), banners, luck: c.five_stars ? luck(c.rules, c.avg_pity, c.five_stars, 3000, 77) : null };
  },
  history(id, bid, min) {
    const cnt = {}, rows = [];
    for (const p of this.pulls(id)) {
      const c = cnt[p.banner] || (cnt[p.banner] = [0, 0]); c[0]++; c[1]++;
      let pity = c[0]; if (p.rarity === 5) { c[0] = 0; c[1] = 0; } else if (p.rarity === 4) { pity = c[1]; c[1] = 0; }
      if (p.banner === bid && p.rarity >= min) rows.push({ ...p, pity });
    }
    return { total: rows.length, rows: rows.reverse() };
  },
};
const api = {
  overview: id => tauri ? invoke('get_overview', { gameId: id }) : mock.overview(id),
  history: (id, b, min) => tauri ? invoke('get_history', { gameId: id, bannerId: b, minRarity: min, limit: 2000 }) : mock.history(id, b, min),
  plan: input => tauri ? invoke('plan_savings', { input }) : plan(input),
};

// ---------- State + routing ----------
const S = { screen: 'overview', game: 'genshin', banner: 'character', ov: null, min: 3, q: '', ringK: 1, glow: 0, flash: -1,
  odds: { pulls: 40, pity: null, guaranteed: null },
  plan: { currency: 6880, pulls_owned: 9, days: 25, daily_missions: true, monthly_pass: true, battle_pass: true, endgame_clear: .9, events_per_patch: 4200 },
  imp: { stage: 4, prog: [1, .64, 0, 0], running: false } };
const SCREENS = [['overview', 'Overview', 'overview'], ['history', 'History', 'history'], ['odds', 'Odds & Planner', 'odds'], ['import', 'Import', 'import']];
const bannerStats = () => S.ov.banners.find(b => b.banner === S.banner) || S.ov.banners[0];

async function load() {
  S.ov = await api.overview(S.game);
  if (!S.ov.banners.some(b => b.banner === S.banner)) S.banner = S.ov.banners[0].banner;
}
async function route() {
  const [screen, game, banner] = location.hash.slice(1).split('/');
  if (SCREENS.some(s => s[0] === screen)) S.screen = screen;
  const changed = game && game !== S.game && GAMES.some(g => g.id === game);
  if (changed) S.game = game;
  if (banner) S.banner = banner;
  if (!S.ov || changed || S.ov.game !== S.game) await load();
  await render();
}
const go = (screen = S.screen, game = S.game, banner = S.banner) => { location.hash = `${screen}/${game}/${banner}`; };

function chrome() {
  const g = G(S.game);
  $('#games').innerHTML = GAMES.map(x => `<button class="${x.id === S.game ? 'on' : ''}" data-game="${x.id}"><i class="mark" style="background:${x.color}"></i>${x.short}</button>`).join('');
  $('#syncState').innerHTML = S.ov.is_sample ? '<span class="tag">Sample data</span>' : '<i class="dot"></i>Synced 2 min ago';
  const ring = (v, max, soft) => { const c = 2 * Math.PI * 8; return `<svg width="22" height="22" viewBox="0 0 22 22"><circle cx="11" cy="11" r="8" fill="none" stroke="var(--paper-3)" stroke-width="3"/><circle cx="11" cy="11" r="8" fill="none" stroke="${soft ? 'var(--gold)' : 'var(--ink)'}" stroke-width="3" stroke-dasharray="${c * v / max} ${c}" transform="rotate(-90 11 11)"/></svg>`; };
  $('#sidebar').innerHTML = `<div class="eyebrow side-h">Views</div>` +
    SCREENS.map(([id, label, icon]) => `<button class="nav ${S.screen === id ? 'on' : ''}" data-screen="${id}"><svg viewBox="0 0 16 16">${I[icon]}</svg>${label}${id === 'history' ? `<span class="k">${num(S.ov.banners.reduce((a, b) => a + b.total, 0))}</span>` : ''}</button>`).join('') +
    `<div class="eyebrow side-h" style="margin-top:12px">Banners</div>` +
    S.ov.banners.map((b, i) => `<button class="bnr ${b.banner === S.banner ? 'on' : ''}" data-banner="${b.banner}">${ring(b.pity, b.rules.hard_pity, b.in_soft_pity)}<span><b>${g.banners[i][2]}</b><small>${b.guaranteed ? 'Guaranteed' : b.rules.featured_rate < 1 ? (b.rules.featured_rate === .5 ? '50/50' : '75/25') : 'No rate-up'} · ${b.rules.hard_pity} hard</small></span><span class="p ${b.in_soft_pity ? 'soft' : ''}">${b.pity}</span></button>`).join('') +
    `<div class="acct"><b>${g.title}</b><span>UID ${S.ov.uid || '—'} · ${g.region}</span></div>`;
}

async function render() {
  chrome();
  const v = $('#view');
  v.innerHTML = await VIEWS[S.screen]();
  v.dataset.screen = S.screen;
  if (S.screen === 'odds') bindOdds();
}

// ---------- Overview ----------
function bigRing(b, size = 248) {
  const r = b.rules, max = r.hard_pity, soft = r.soft_start - 1, v = b.pity * S.ringK, cx = size / 2, rad = cx - 30, C = 2 * Math.PI * rad;
  const arc = (from, to, stroke, w, extra = '') => `<circle cx="${cx}" cy="${cx}" r="${rad}" fill="none" stroke="${stroke}" stroke-width="${w}" stroke-dasharray="${C * (to - from) / max} ${C}" stroke-dashoffset="${-C * from / max}" transform="rotate(-90 ${cx} ${cx})" ${extra}/>`;
  let ticks = '';
  for (let k = 0; k < max; k += 10) {
    const a = k / max * 2 * Math.PI - Math.PI / 2, o = rad + 12, i = rad + (k % 30 ? 15 : 18);
    ticks += `<line x1="${cx + Math.cos(a) * o}" y1="${cx + Math.sin(a) * o}" x2="${cx + Math.cos(a) * i}" y2="${cx + Math.sin(a) * i}" stroke="var(--ink-4)" stroke-width="1"/>`;
  }
  const lab = (k, t, col) => { const a = k / max * 2 * Math.PI - Math.PI / 2, d = rad + 27; return `<text x="${cx + Math.cos(a) * d}" y="${cx + Math.sin(a) * d + 3.5}" text-anchor="middle" font-size="10" font-weight="600" fill="${col}">${t}</text>`; };
  const goldV = Math.max(0, v - soft), inSoft = b.pity >= soft;
  return `<div class="ringbox ${inSoft ? 'soft' : ''}"><svg width="${size}" height="${size}" style="overflow:visible">
    <defs><filter id="glow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="5"/></filter></defs>
    ${arc(0, max, 'var(--paper-3)', 12)}${arc(soft, max, 'rgba(184,137,43,.18)', 12)}${arc(soft, max, 'var(--gold)', 2, 'opacity=".55"').replace(`r="${rad}"`, `r="${rad + 9}"`)}
    ${ticks}${lab(0, '0', 'var(--ink-3)')}${lab(30, '30', 'var(--ink-3)')}${lab(60, '60', 'var(--ink-3)')}${lab(soft + 1, soft + 1, 'var(--gold)')}
    ${arc(0, Math.min(v, soft), 'var(--ink)', 12)}
    ${goldV > 0 ? arc(soft, soft + goldV, 'var(--gold)', 14, `filter="url(#glow)" opacity="${.35 + S.glow * .5}"`) + arc(soft, soft + goldV, 'var(--gold)', 12) : ''}
  </svg><div class="c"><div class="eyebrow">5★ pity</div><div class="big">${Math.round(v)}<small>/${max}</small></div>
  <div class="sub">${inSoft ? `<span class="gold-t">Soft pity</span> · ${max - b.pity} to hard` : `Soft pity in ${soft + 1 - b.pity}`}</div></div></div>`;
}
function luckChart(L, w = 452, h = 128) {
  if (!L) return '<p class="muted">No 5★ yet on this banner.</p>';
  const lo = 40, hi = 82, x = m => 34 + (m - lo) / (hi - lo) * (w - 44), tot = L.samples, mx = Math.max(...L.bins) / tot, y = s => h - 22 - s / mx * (h - 40);
  let bars = '';
  L.bins.forEach((c, i) => {
    const m = L.bin_start + i * L.bin_width; if (m < lo || m >= hi) return;
    bars += `<rect x="${x(m) + 1}" y="${y(c / tot)}" width="${x(m + 2) - x(m) - 2}" height="${h - 22 - y(c / tot)}" fill="${m + 2 <= L.observed ? 'var(--paper-3)' : 'rgba(184,137,43,.32)'}"/>`;
  });
  let ax = '';
  for (let m = lo; m <= hi; m += 6) ax += `<text x="${x(m)}" y="${h - 6}" text-anchor="middle">${m}</text>`;
  for (let k = 1; k <= 2; k++) { const s = mx * k / 2; ax += `<line class="grid" x1="34" x2="${w - 10}" y1="${y(s)}" y2="${y(s)}"/><text x="28" y="${y(s) + 3.5}" text-anchor="end">${(s * 100).toFixed(0)}%</text>`; }
  const ox = x(L.observed), ex = x(L.expected), sw = L.observed > L.expected;
  return `<svg class="chart" width="${w}" height="${h}">${ax}${bars}<line class="ax" x1="34" x2="${w - 10}" y1="${h - 22}" y2="${h - 22}"/>
    <line x1="${ex}" x2="${ex}" y1="10" y2="${h - 22}" stroke="var(--ink-3)" stroke-dasharray="3 3"/><text x="${ex + (sw ? -5 : 5)}" y="12" text-anchor="${sw ? 'end' : 'start'}">Expected ${L.expected.toFixed(1)}</text>
    <line x1="${ox}" x2="${ox}" y1="4" y2="${h - 22}" stroke="var(--gold)" stroke-width="1.5"/><text x="${ox + (sw ? 5 : -5)}" y="12" text-anchor="${sw ? 'start' : 'end'}" style="fill:var(--gold);font-weight:600">You ${L.observed.toFixed(1)}</text></svg>`;
}
const OUT = { won: ['Won 50/50', 'gold'], lost: ['Lost 50/50', ''], guaranteed: ['Guaranteed', 'gold'], standard: ['Standard', ''] };
const VIEWS = {
  overview() {
    const g = G(S.game), b = bannerStats(), r = b.rules, last = b.hits.at(-1), luckOn = S.banner === S.ov.banners[0].banner;
    const p10 = pAny(r, b.pity, 10), ff = r.featured_rate < 1;
    const hits = b.hits.slice(-9), segs = hits.map((h, i) => `<div class="seg ${h.outcome} ${i === S.flash ? 'flash' : ''}" style="flex:${h.pity}">${h.pity}</div>`).join('') + `<div class="seg current" style="flex:${Math.max(b.pity, 9)}">${b.pity}</div>`;
    const labels = hits.map(h => `<div style="flex:${h.pity}"><b>${h.name}</b>${fdate(h.time)}</div>`).join('') + `<div style="flex:${Math.max(b.pity, 9)}"><b>Next</b>in progress</div>`;
    const fact = (label, v, d) => `<div class="fact"><div class="eyebrow">${label}</div><div class="v">${v}</div><div class="d">${d}</div></div>`;
    const L = luckOn ? S.ov.luck : null;
    return `<div class="head"><div><div class="eyebrow">${g.title} · ${b.label}</div><h1>${num(b.total)} ${g.word} <em>· ${b.five_stars} five-stars</em></h1></div>
      <div class="meta">${last ? `Last 5★ <b>${last.name}</b> · ${fdate(last.time)}<br>` : ''}UID ${S.ov.uid} · ${g.region}</div></div>
    <div class="ov-top"><div>${bigRing(b)}</div><div>
      <div class="facts">
        ${fact('Guarantee', b.guaranteed ? '<span class="gold-t">Guaranteed</span>' : ff ? (r.featured_rate === .5 ? '50/50' : '75/25') : 'None', b.guaranteed ? 'Lost the last 50/50' : ff ? 'Next 5★ may be off-banner' : 'Banner has no rate-up split')}
        ${fact('Next 10 pulls', `${pct(p10)}`, 'Chance of at least one 5★')}
        ${fact('Average pity', b.avg_pity ? b.avg_pity.toFixed(1) : '—', `Expected ${expected(r).toFixed(1)} for this banner`)}
        ${fact('50/50 record', ff ? `${b.wins}<small>won</small> ${b.losses}<small>lost</small>` : '—', b.win_rate != null ? `${pct(b.win_rate, 0)} win rate` : 'No 50/50 on this banner')}
        ${fact('4★ pulled', `${b.four_stars}<small>${(b.four_stars / Math.max(1, b.total) * 100).toFixed(1)}%</small>`, `4★ pity <span class="violet-t">${b.pity4}</span> of 10`)}
        ${fact('Spent', num(b.total * 160), g.cur)}
      </div>
      <div class="notes">${b.notes.map(n => `<div>${n}</div>`).join('')}</div>
    </div></div>
    <div class="sec"><span class="eyebrow">Five-star timeline · width is pity</span><span class="legend"><span><i style="background:var(--gold)"></i>Won</span><span><i style="background:var(--ink-4)"></i>Lost</span><span><i style="border:1px solid var(--gold)"></i>Guaranteed</span><span><i style="border:1px dashed var(--ink-4)"></i>Current</span></span></div>
    <div class="strip">${segs}</div><div class="strip-l">${labels}</div>
    <div class="cols"><div><div class="sec"><span class="eyebrow">Luck percentile</span><span class="r">${L ? `${num(L.samples)} simulated players, ${b.five_stars} five-stars each` : ''}</span></div>
      ${L ? `<div class="callout"><span class="v">${L.percentile.toFixed(0)}%</span><span class="muted">of simulated players had a worse average pity than you</span></div>` : ''}${luckOn ? luckChart(L) : '<p class="muted">Luck percentile is computed for the character banner.</p>'}</div>
      <div><div class="sec"><span class="eyebrow">Five-star log</span><span class="r">${b.five_stars} total</span></div>
      <table class="list">${b.hits.slice(-5).reverse().map(h => `<tr><td class="muted" style="width:62px">${fdate(h.time)}</td><td class="nm">${h.name}</td><td style="width:70px" class="tnum">pity <b>${h.pity}</b></td><td style="width:96px"><span class="tag ${OUT[h.outcome][1]}">${OUT[h.outcome][0]}</span></td></tr>`).join('')}</table></div></div>`;
  },

  // ---------- History ----------
  async history() {
    const g = G(S.game), b = bannerStats(), page = await api.history(S.game, S.banner, S.min), q = S.q.toLowerCase();
    const rows = page.rows.filter(r => !q || r.name.toLowerCase().includes(q)).slice(0, 160), max = b.rules.hard_pity;
    const outByTime = Object.fromEntries(b.hits.map(h => [h.time + h.name, h.outcome]));
    return `<div class="head"><div><div class="eyebrow">${g.title} · ${b.label}</div><h1>Pull history <em>· ${num(page.total)} ${S.min > 3 ? 'shown' : g.word}</em></h1></div><div class="meta">Newest first<br>Times in server time (UTC+1)</div></div>
    <div class="toolbar"><div class="seg-ctl">${[[3, 'All'], [4, '4★ and up'], [5, '5★ only']].map(([m, l]) => `<button class="${S.min === m ? 'on' : ''}" data-min="${m}">${l}</button>`).join('')}</div>
      <label class="field">${ico('search')}<input id="q" placeholder="Search items" value="${S.q}"></label><div class="spacer"></div>
      <button class="btn" data-act="export">${ico('export')}Export UIGF</button></div>
    <table class="tbl"><thead><tr><th style="width:150px">Time</th><th>Item</th><th style="width:110px">Type</th><th style="width:80px">Rarity</th><th style="width:150px">Pity at pull</th><th style="width:120px">Result</th></tr></thead><tbody>
    ${rows.map(r => { const o = r.rarity === 5 ? outByTime[r.time + r.name] : null, m = r.rarity === 4 ? 10 : max;
      return `<tr class="r${r.rarity}"><td class="t">${r.time}</td><td class="nm">${r.name}</td><td class="muted">${r.item_type}</td><td class="stars">${'★'.repeat(r.rarity)}</td>
      <td><div class="pbar"><b>${r.pity}</b><i><span style="width:${Math.min(100, r.pity / m * 100)}%"></span></i></div></td><td>${o ? `<span class="tag ${OUT[o][1]}">${OUT[o][0]}</span>` : ''}</td></tr>`; }).join('')}
    </tbody></table>`;
  },

  // ---------- Odds + planner ----------
  async odds() {
    const g = G(S.game), b = bannerStats(), r = b.rules, o = S.odds, pity = o.pity ?? b.pity, gua = o.guaranteed ?? b.guaranteed, n = o.pulls;
    const pf = pFeatured(r, pity, gua, n), pa = pAny(r, pity, n), n90 = forConf(r, pity, gua, .9);
    const P = { ...S.plan, game: S.game, banner: S.banner, pity, guaranteed: gua }, res = await api.plan(P);
    const w = 600, h = 250, X = k => 40 + k / (r.hard_pity * 2) * (w - 54), Y = p => 14 + (1 - p) * (h - 44);
    let pa_ = '', pf_ = '', grid = '';
    for (let k = 0; k <= r.hard_pity * 2; k += 2) { pa_ += `${k ? 'L' : 'M'}${X(k).toFixed(1)} ${Y(pAny(r, pity, k)).toFixed(1)}`; pf_ += `${k ? 'L' : 'M'}${X(k).toFixed(1)} ${Y(pFeatured(r, pity, gua, k)).toFixed(1)}`; }
    for (let p = 0; p <= 1; p += .25) grid += `<line class="grid" x1="40" x2="${w - 14}" y1="${Y(p)}" y2="${Y(p)}"/><text x="32" y="${Y(p) + 3.5}" text-anchor="end">${p * 100}%</text>`;
    for (let k = 0; k <= r.hard_pity * 2; k += 20) grid += `<text x="${X(k)}" y="${h - 12}" text-anchor="middle">${k}</text>`;
    const s0 = Math.max(0, r.soft_start - pity - 1), s1 = r.hard_pity - pity;
    const chart = `<svg class="chart" width="${w}" height="${h}"><rect x="${X(s0)}" y="14" width="${X(s1) - X(s0)}" height="${h - 44}" fill="var(--gold-soft)"/><text x="${X(s0) + 6}" y="28" style="fill:#8E6716;font-weight:600">Soft pity</text>${grid}
      <line class="ax" x1="40" x2="${w - 14}" y1="${Y(0)}" y2="${Y(0)}"/><path d="${pa_}" fill="none" stroke="var(--ink)" stroke-width="1.6"/><path d="${pf_}" fill="none" stroke="var(--gold)" stroke-width="2.2"/>
      <line x1="${X(n)}" x2="${X(n)}" y1="14" y2="${Y(0)}" stroke="var(--ink-2)" stroke-dasharray="3 3"/><circle cx="${X(n)}" cy="${Y(pa)}" r="4" fill="var(--card)" stroke="var(--ink)" stroke-width="1.6"/><circle cx="${X(n)}" cy="${Y(pf)}" r="4.5" fill="var(--gold)" stroke="var(--card)" stroke-width="1.5"/>
      <text x="${X(n) + (n > 150 ? -8 : 8)}" y="${Y(0) - 8}" text-anchor="${n > 150 ? 'end' : 'start'}" style="fill:var(--ink);font-weight:600">${n} pulls</text></svg>`;
    const tog = (k, label, sub) => `<div class="in"><span><b>${label}</b> <small class="muted">· ${sub}</small></span><button class="switch ${S.plan[k] ? 'on' : ''}" data-tog="${k}"></button></div>`;
    const inp = (k, label) => `<div class="in"><b>${label}</b><label class="field"><input data-plan="${k}" value="${num(S.plan[k])}"></label></div>`;
    const target = new Date(Date.UTC(2026, 10, 8) + S.plan.days * 864e5);
    return `<div class="head"><div><div class="eyebrow">${g.title} · ${b.label}</div><h1>Odds <em>&amp; savings planner</em></h1></div><div class="meta">Closed-form from the soft-pity curve<br>Base ${(r.base_rate * 100).toFixed(1)}% · soft pity ${r.soft_start} · hard ${r.hard_pity}</div></div>
    <div class="odds"><div>
      <div class="kpis"><div><div class="eyebrow">Any 5★ within ${n}</div><div class="v" id="kAny">${pct(pa)}</div></div><div><div class="eyebrow">Featured within ${n}</div><div class="v gold-t" id="kFeat">${pct(pf)}</div></div><div><div class="eyebrow">Pulls for 90% featured</div><div class="v">${n90}</div></div></div>
      <div class="sec"><span class="eyebrow">Cumulative probability</span><span class="legend"><span><i style="background:var(--ink)"></i>Any 5★</span><span><i style="background:var(--gold)"></i>Featured 5★</span></span></div>${chart}
      <div class="ctl"><label for="pulls">Planned pulls</label><span class="val">${n} · ${num(n * 160)} ${g.cur}</span><input type="range" id="pulls" min="0" max="${r.hard_pity * 2}" value="${n}" style="--f:${n / (r.hard_pity * 2) * 100}%"></div>
      <div class="ctl"><label>Current pity</label><span class="stepper"><button data-pity="-1">−</button><span>${pity}</span><button data-pity="1">+</button></span><span class="hint">Taken from your history: ${b.pity}. Adjust to explore.</span></div>
      <div class="ctl"><label>Guaranteed featured</label><button class="switch ${gua ? 'on' : ''}" data-act="gua"></button><span class="hint">On after losing a 50/50. ${r.capturing_radiance ? 'Capturing Radiance is not modelled in the curve.' : ''}</span></div>
    </div>
    <div class="plan"><div class="sec" style="margin-bottom:4px"><span class="eyebrow">Savings planner</span><span class="r">${S.plan.days} days</span></div>
      ${inp('currency', g.cur)}${inp('pulls_owned', g.item)}
      <div class="in"><b>Target banner</b><span class="tnum">Next version · ${MON[target.getUTCMonth()]} ${target.getUTCDate()}</span></div>
      ${tog('daily_missions', 'Daily missions', '60 per day')}${tog('monthly_pass', g.pass, '90 per day')}${tog('battle_pass', 'Battle pass', '4 pulls per version')}
      <div class="tot"><div><div class="eyebrow">Pulls by ${MON[target.getUTCMonth()]} ${target.getUTCDate()}</div><div class="v">${res.pulls}</div></div><div style="text-align:right"><div class="eyebrow">Featured chance</div><div class="v gold-t" style="font-size:30px">${pct(res.p_featured, 0)}</div></div></div>
      <table>${res.lines.map(l => `<tr><td>${l.label}</td><td>${num(l.amount)}</td></tr>`).join('')}<tr><td><b>Total</b></td><td><b>${num(res.currency_total)}</b></td></tr></table>
      <p class="muted" style="margin-top:10px;font-size:12px">${res.shortfall_90 ? `${res.shortfall_90} more pulls for a 90% featured chance.` : 'You are above 90% for the featured 5★.'}</p>
    </div></div>`;
  },

  // ---------- Import ----------
  import() {
    const g = G(S.game), st = S.imp.stage, cache = g.dir ? `${g.dir}\\webCaches\\2.38.0.0\\Cache\\Cache_Data\\data_2` : '';
    const steps = [['Game log', g.log, '2.1 MB'], ...(g.dir ? [['Game folder', g.dir, 'from log'], ['Web cache', cache, '18.6 MB scanned']] : []),
      ['History URL', `https://${g.host}?authkey=d0J5Vx...(1024 chars)&lang=en`, 'valid ~24 h']];
    const fetchI = steps.length;
    const step = (i, [t, p, r]) => { const s = i < st ? 'ok' : i === st ? 'run' : 'wait'; return `<div class="step ${s}"><span class="b">${s === 'ok' ? `<svg viewBox="0 0 16 16">${I.check}</svg>` : i + 1}</span><div><b>${t}</b><div class="path">${s === 'wait' ? 'Waiting' : p}</div></div><span class="r">${s === 'wait' ? '' : r}</span></div>`; };
    const totals = S.ov.banners.map(b => b.total);
    const fetch = `<div class="step ${st > fetchI ? 'ok' : st === fetchI ? 'run' : 'wait'}"><span class="b">${st > fetchI ? `<svg viewBox="0 0 16 16">${I.check}</svg>` : fetchI + 1}</span><div><b>Download history</b><div style="margin-top:8px">
      ${S.ov.banners.map((b, i) => { const p = st < fetchI ? 0 : st > fetchI ? 1 : S.imp.prog[i] || 0; return `<div class="fetch"><span>${b.label}</span><div class="bar"><span style="width:${p * 100}%"></span></div><span class="n">${num(totals[i] * p)} / ${num(totals[i])}</span></div>`; }).join('')}
      </div></div><span class="r">${st === fetchI ? 'page ' + Math.max(1, Math.round(S.imp.prog.reduce((a, b) => a + b, 0) * 14)) : ''}</span></div>`;
    return `<div class="head"><div><div class="eyebrow">${g.title} · Read-only import</div><h1>One-click import <em>· no login</em></h1></div><div class="meta">Reads the game's own log and web cache<br>Nothing in the game folder is modified</div></div>
    <div class="imp"><div><div class="steps">${steps.map((s, i) => step(i, s)).join('')}${fetch}</div>
      <div style="display:flex;gap:10px;margin-top:18px"><button class="btn primary" data-act="import">${ico('import')}Import ${g.word}</button><button class="btn" data-act="rescan">${ico('refresh')}Scan again</button></div>
      <p class="muted" style="margin-top:12px;max-width:560px">Open the ${g.word.slice(0, -1).replace('wishe', 'wish')} history page in-game once, then press Import. The link expires after about a day; Wishboard only stores the pulls.</p></div>
    <div><div class="panel"><div class="eyebrow">UIGF v4.0</div><p>Move your history between trackers. Exports Genshin, Star Rail and ZZZ as standard UIGF sections; Wuthering Waves goes into an extension block.</p>
      <div style="display:flex;gap:8px"><button class="btn" data-act="export">${ico('export')}Export JSON</button><button class="btn">${ico('file')}Open file</button></div><div class="drop">Drop a .json export here</div></div>
      <div class="panel"><div class="eyebrow">Detected games</div>${GAMES.map(x => `<div class="gamerow"><i class="mark" style="background:${x.color}"></i>${x.title}<span>${x.id === 'wuwa' ? 'Client.log' : 'Player log'} found</span></div>`).join('')}</div></div></div>`;
  },
};

// ---------- Events ----------
function bindOdds() {
  const s = $('#pulls'); if (!s) return;
  s.oninput = () => { S.odds.pulls = +s.value; render(); requestAnimationFrame(() => $('#pulls')?.focus()); };
}
document.addEventListener('click', async e => {
  const t = e.target.closest('button'); if (!t) return;
  const d = t.dataset;
  if (d.win && tauri) return tauri.window.getCurrentWindow()[d.win]();
  if (d.game) { S.odds.pity = S.odds.guaranteed = null; return go(S.screen, d.game, 'character'); }
  if (d.screen) return go(d.screen);
  if (d.banner) { S.odds.pity = S.odds.guaranteed = null; return go(S.screen, S.game, d.banner); }
  if (d.min) { S.min = +d.min; return render(); }
  if (d.tog) { S.plan[d.tog] = !S.plan[d.tog]; return render(); }
  if (d.pity) { const b = bannerStats(); S.odds.pity = Math.max(0, Math.min(b.rules.hard_pity - 1, (S.odds.pity ?? b.pity) + +d.pity)); return render(); }
  if (d.act === 'gua') { S.odds.guaranteed = !(S.odds.guaranteed ?? bannerStats().guaranteed); return render(); }
  if (d.act === 'export' && tauri) { const r = await invoke('export_uigf', {}); return alert(`Saved ${r.records} pulls to ${r.path}`); }
  if (d.act === 'import' && !S.imp.running) {
    if (tauri) { try { await invoke('start_import', { gameId: S.game }); } catch (err) { alert(err); } return; }
    S.imp = { stage: 0, prog: [0, 0, 0, 0], running: true };
    const n = G(S.game).dir ? 4 : 2, t0 = performance.now();
    const tick = () => { const k = (performance.now() - t0) / 500; S.imp.stage = Math.min(n + 1, Math.floor(k)); if (S.imp.stage >= n) S.imp.prog = S.imp.prog.map((_, i) => Math.max(0, Math.min(1, (k - n) * 1.5 - i * .5))); render(); if (k < n + 3) requestAnimationFrame(tick); else { S.imp.running = false; go('overview'); } };
    tick();
  }
});
document.addEventListener('input', e => {
  if (e.target.id === 'q') { S.q = e.target.value; clearTimeout(S.qt); S.qt = setTimeout(async () => { await render(); const q = $('#q'); q.focus(); q.setSelectionRange(q.value.length, q.value.length); }, 200); }
  if (e.target.dataset.plan) { S.plan[e.target.dataset.plan] = +e.target.value.replace(/\D/g, '') || 0; }
});
document.addEventListener('change', e => { if (e.target.dataset.plan) render(); });
window.addEventListener('hashchange', route);
window.WB = { S, render, route, go, bannerStats };
route();
