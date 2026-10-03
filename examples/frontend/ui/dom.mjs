export function el(tag,className,text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = String(text);
  return node;
}
export function button(text,className,onClick) {
  const node = el('button',className,text); node.type = 'button';
  if (onClick) node.addEventListener('click',onClick); return node;
}
export function table(columns,rows,{limit = Infinity,onMore,empty = 'Không có dữ liệu trong payload.'} = {}) {
  const wrap = el('div','astro-table-wrap');
  if (!rows.length) { wrap.append(el('p','astro-empty',empty)); return wrap; }
  const t = el('table','astro-table'); const head = el('thead'); const heading = el('tr');
  for (const column of columns) { const h = el('th',column.className || '',column.label); h.scope = 'col'; heading.append(h); }
  head.append(heading); t.append(head); const body = el('tbody');
  for (const row of rows.slice(0,limit)) {
    const tr = el('tr');
    for (const column of columns) { const td = el('td',column.className || ''); const value = column.render(row);
      if (value?.nodeType) td.append(value); else td.textContent = value === undefined || value === null ? '—' : String(value);
      tr.append(td);
    } body.append(tr);
  } t.append(body); wrap.append(t);
  if (rows.length > limit) {
    const footer = el('div','astro-table-more');footer.append(el('span','astro-muted',`Hiển thị ${limit} / ${rows.length}`));
    if (onMore) footer.append(button('Xem thêm','astro-small-button',onMore)); wrap.append(footer);
  }
  return wrap;
}
export function boundedJson(value,limit = 24000) {
  const container = el('div','astro-json-block');
  const text = JSON.stringify(value,null,2) ?? 'null'; const pre = el('pre','astro-json'); pre.tabIndex = 0;
  const code = el('code','',text.length > limit ? `${text.slice(0,limit)}\n…` : text); pre.append(code); container.append(pre);
  if (text.length > limit) container.append(el('p','astro-muted astro-json-note',`Bản xem rút gọn ${limit.toLocaleString()} / ${text.length.toLocaleString()} ký tự. Payload gốc được giữ nguyên; dùng tải JSON để lấy toàn bộ.`));
  return container;
}
export function disclosure(label,value,{jsonLimit = 6000} = {}) {
  const details = el('details','astro-evidence'); details.append(el('summary','',label));
  details.addEventListener('toggle',() => { if (details.open && details.children.length === 1) details.append(boundedJson(value,jsonLimit)); });
  return details;
}
