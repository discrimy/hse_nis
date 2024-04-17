local utf8 = require(".utf8.init"):init()

function serializeTable(val, name, depth)
    depth = depth or 0

    local tmp = string.rep(" ", depth)

    if name then tmp = tmp .. name .. " = " end

    if type(val) == "table" then
        tmp = tmp .. "{"

        for k, v in pairs(val) do
            tmp =  tmp .. serializeTable(v, k, depth + 1) .. ","
        end

        tmp = tmp .. string.rep(" ", depth) .. "}"
    elseif type(val) == "number" then
        tmp = tmp .. tostring(val)
    elseif type(val) == "string" then
        tmp = tmp .. string.format("%q", val)
    elseif type(val) == "boolean" then
        tmp = tmp .. (val and "true" or "false")
    else
        tmp = tmp .. "\"[inserializeable datatype:" .. type(val) .. "]\""
    end

    return tmp
end

-- return a new array containing the concatenation of all of its 
-- parameters. Scaler parameters are included in place, and array 
-- parameters have their values shallow-copied to the final array.
-- Note that userdata and function values are treated as scalar.
function array_concat(...) 
    local t = {}
    for n = 1,select("#",...) do
        local arg = select(n,...)
        if type(arg)=="table" then
            for _,v in ipairs(arg) do
                t[#t+1] = v
            end
        else
            t[#t+1] = arg
        end
    end
    return t
end

function table.all(t, predicate)
  for _, value in pairs(t) do
    if not predicate(value) then
      return false
    end
  end
  return true
end

function table.any(t, predicate)
  for _, value in pairs(t) do
    if predicate(value) then
      return true
    end
  end
  return false
end

local instructions = {}
local line = io.read("*L")
while line do
  line = line:gsub("^%s*(.-)%s*$", "%1")
  table.insert(instructions, line)
  line = io.read("*L")
end

local persons = {}
local graph = {}
local couples = {}
for _, line in pairs(instructions) do
  local _, _, name, sex = utf8.find(line, "(.+) %(([М|Ж])%)")
  if name then
    persons[name] = sex
  else 
    local _, _, name1, conn, name2 = utf8.find(line, "(.+) (.+) (.+)")
    if name1 ~= nil then
      if conn == '<->' then
        couples[name1] = name2
        couples[name2] = name1
      end

      if graph[name1] == nil then graph[name1] = {} end
      if graph[name2] == nil then graph[name2] = {} end
      graph[name1][name2] = conn
      local rev_conn = (conn == '->') and '<-' or '<->'
      graph[name2][name1] = rev_conn

      -- Исправление, что родителя у детей указываются не все
      if conn == '->' and couples[name1] then
        local name1 = couples[name1]

        if graph[name1] == nil then graph[name1] = {} end
        if graph[name2] == nil then graph[name2] = {} end
        graph[name1][name2] = '->'
        graph[name2][name1] = '<-'
      end
    end
  end
end
-- for name, rels in pairs(graph) do
--   print(name .. ": " .. serializeTable(rels))
-- end

-- Функция для рекурсивного поиска кратчайшего пути от источника до цели
function dijkstra(graph, source, target)
  -- Инициализация расстояний и посещенных узлов
  local distances = {}
  local visited = {}
  for node in pairs(graph) do
      distances[node] = math.huge
      visited[node] = false
  end
  distances[source] = 0

  -- Таблица для хранения пути
  local paths = {}

  -- Очередь с приоритетами для выбора следующего узла
  local queue = {source}

  while #queue > 0 do
      -- Извлекаем узел с минимальным расстоянием
      local u = table.remove(queue, 1)
      visited[u] = true

      -- Перебираем соседей текущего узла
      for v, conn in pairs(graph[u]) do
          if not visited[v] then
              -- Обновляем расстояние, если найден более короткий путь
              local current_distance = distances[u] + 1
              if current_distance < distances[v] then
                  distances[v] = current_distance
                  paths[v] = u
                  -- Добавляем узел обратно в очередь, чтобы повторно обработать его
                  table.insert(queue, v)
              end
          end
          visited[v] = true
      end
  end

  -- Восстанавливаем путь
  local path = {}
  local current = {name = target}
  while current.name ~= source do
      path[#path+1] = current
      current = {name = paths[current.name]}
  end

  local prev = source
  for i = #path, 1, -1 do
    path[i].conn = graph[prev][path[i].name]
    path[i].sex = persons[path[i].name]
    prev = path[i].name
  end

  local result = {}
  for _, item in pairs(path) do
    table.insert(result, 1, item)
  end

  -- Проверяем, достигнут ли целевой узел
  return #result > 0 and result or nil, distances
end



-- local from = "Парфений"
-- local to = "Архипп"
local from, to = ...

print('Search path from ' .. from .. ' to ' .. to)
local path, distances = dijkstra(graph, from, to)
-- print(serializeTable(path))
local path_verbose = from .. ' (' .. persons[from] .. ')'
for _, item in pairs(path) do
  path_verbose = path_verbose .. ' ' .. item.conn .. ' ' .. item.name .. ' (' .. item.sex .. ')' 
end
print(path_verbose)

-- Определение родства
local role = 'Неизвестно'
if #path == 1 and path[1].conn == '->' and path[1].sex == 'Ж' then
  role = 'Дочь'
elseif #path == 1 and path[1].conn == '->' and path[1].sex == 'М' then
  role = 'Сын'
elseif #path > 1 and table.all(path, function (item) return item.conn == '->' end) then
  -- Xвнук/внучка
  local sex = path[#path].sex
  local pra_n = #path - 2
  if pra_n == 0 then
    if sex == 'М' then
      role = 'Внук'
    else
      role = 'Внучка'
    end
  else
    local infix = string.rep('пра', pra_n - 1)
    if sex == 'М' then
      role = 'Пра' .. infix .. 'внук'
    else
      role = 'Пра' .. infix .. 'внучка'
    end
  end
elseif #path == 1 and path[1].conn == '<->' and path[1].sex == 'Ж' then
  role = 'Жена'
elseif #path == 1 and path[1].conn == '<->' and path[1].sex == 'М' then
  role = 'Муж'
elseif #path == 1 and path[1].conn == '<-' and path[1].sex == 'Ж' then
  role = 'Мать'
elseif #path == 1 and path[1].conn == '<-' and path[1].sex == 'М' then
  role = 'Отец'
end
print(role)