import io
import re
import glob

def read(p):
    return io.open(p, 'r', encoding='utf-8').read().replace('\r\n', '\n')

def brace_end(lines, i):
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
            return j
        j += 1
    return len(lines) - 1

for f in glob.glob('crates/sdkwork-game-*-repository-sqlx/src/sqlx.rs'):
    c = read(f)
    lines = c.split('\n')
    out = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if re.match(r'DatabasePool::Sqlite\((\w+), _\) => \{', stripped):
            end = brace_end(lines, i)
            out.append('            DatabasePool::Sqlite(_, _) => {')
            out.append('                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)")')
            out.append('            }')
            i = end + 1
            continue
        if re.match(r'DatabasePool::Sqlite\((\w+), _\) => .+,\s*$', stripped):
            out.append('            DatabasePool::Sqlite(_, _) =>')
            out.append('                unreachable!("game repository requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)"),')
            i += 1
            continue
        if re.match(r'(async )?fn \w+_sqlite\(', stripped):
            end = brace_end(lines, i)
            i = end + 1
            continue
        out.append(lines[i])
        i += 1
    io.open(f, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))
    name = f.replace('\\', '/').split('/')[-2]
    print(name, 'done; sqlite left:', len(re.findall(r'[sS]qlite', '\n'.join(out))))
