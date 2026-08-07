import io
import re
import glob

def read(p):
    return io.open(p, 'r', encoding='utf-8').read().replace('\r\n', '\n')

def write(p, c):
    io.open(p, 'w', encoding='utf-8', newline='\n').write(c)

GATE = '''    let Some(database_url) = optional_postgres_database_url() else {
        eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");
        return None;
    };'''

POOL = '''        let pool = create_pool_from_config(DatabaseConfig {
            engine: DatabaseEngine::Postgres,
            url: database_url,
            max_connections: 1,
            ..Default::default()
        })
        .await
        .unwrap();'''

HELPER = '''fn optional_postgres_database_url() -> Option<String> {
    std::env::var("SDKWORK_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .ok()
        .filter(|url| url.starts_with("postgres://") || url.starts_with("postgresql://"))
}
'''

def convert(path):
    c = read(path)
    lines = c.split('\n')
    out = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        m = re.match(r'async fn sqlite_repo\(\) -> (\w+) \{', stripped)
        if m:
            ret = m.group(1)
            # find body end
            j = i
            brace = 0
            found = False
            while j < len(lines):
                if not found:
                    if '{' in lines[j]:
                        found = True
                        brace = lines[j].count('{') - lines[j].count('}')
                    j += 1
                    continue
                brace += lines[j].count('{') - lines[j].count('}')
                if brace <= 0:
                    break
                j += 1
            body = lines[i + 1:j]
            # replace sqlite pool config
            body_text = '\n'.join(body)
            body_text = re.sub(r'let pool = create_pool_from_config\(DatabaseConfig \{\n\s*engine: DatabaseEngine::Sqlite,\n\s*url: "sqlite::memory:".into\(\),\n\s*max_connections: 1,\n\s*\.\.Default::default\(\)\n\s*\}\)\n\s*\.await\n\s*\.unwrap\(\);',
                               POOL, body_text)
            # wrap tail expression in Some(...)
            body_lines = body_text.split('\n')
            # find last non-empty, non-brace line before closing
            tail_idx = len(body_lines) - 1
            while tail_idx >= 0 and body_lines[tail_idx].strip() in ('', '}'):
                tail_idx -= 1
            if tail_idx >= 0:
                tail = body_lines[tail_idx].strip()
                if not tail.startswith('Some('):
                    body_lines[tail_idx] = '        Some(' + tail + ')'
            out.append('    async fn postgres_repo() -> Option<' + ret + '> {')
            out.append(GATE)
            out.extend(body_lines)
            out.append('    }')
            i = j + 1
            continue
        # call sites
        m2 = re.match(r'let (\w+) = sqlite_repo\(\)\.await;', stripped)
        if m2:
            out.append('let Some(' + m2.group(1) + ') = postgres_repo().await else {')
            out.append('    eprintln!("skipping game repository test: set SDKWORK_DATABASE_URL or DATABASE_URL to a postgres URL");')
            out.append('    return;')
            out.append('};')
            i += 1
            continue
        # test fn names sqlite_ -> postgres_
        if stripped.startswith('fn sqlite_'):
            out.append(lines[i].replace('fn sqlite_', 'fn postgres_'))
            i += 1
            continue
        out.append(lines[i])
        i += 1
    c = '\n'.join(out)
    if 'fn optional_postgres_database_url' not in c:
        # insert after module tests opening
        idx = c.find('mod tests {')
        if idx >= 0:
            insert_at = c.find('\n', idx) + 1
            c = c[:insert_at] + '\n' + HELPER + c[insert_at:]
    write(path, c)
    print(path.split('\\')[-2], 'converted; sqlite left:', len(re.findall(r'[sS]qlite', c)))

for f in glob.glob('crates/sdkwork-game-*-repository-sqlx/src/sqlx.rs'):
    convert(f)
