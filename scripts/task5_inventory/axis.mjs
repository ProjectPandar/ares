export function axisDefinitions(source, numberList) {
  const clean = stripComments(source);
  const structure = /struct AxisDefault\s*\{([\s\S]*?)\};/.exec(clean)?.[1];
  if (!structure) throw new Error("missing AxisDefault structure");
  const members = [...structure.matchAll(/std::vector<double>\s+([A-Za-z_]+)\s*;/g)].map(match => match[1]);
  if (members.join(",") !== "max_feedrate,max_acceleration,max_jerk") {
    throw new Error(`wrong AxisDefault member order ${members.join(",")}`);
  }
  const table = /std::vector<AxisDefault> axes\s*\{([\s\S]*?)\n\s*\};/.exec(clean);
  if (!table) throw new Error("missing axis defaults");
  const loopStart = clean.indexOf("for (const AxisDefault &axis : axes)", table.index);
  const loopEnd = clean.indexOf("\n        }\n", loopStart);
  if (loopStart < 0 || loopEnd < 0) throw new Error("missing axis registration loop");
  const loop = clean.slice(loopStart, loopEnd);
  const groups = [
    ["machine_max_speed_", "max_feedrate"],
    ["machine_max_acceleration_", "max_acceleration"],
    ["machine_max_jerk_", "max_jerk"],
  ];
  const registrations = groups.map(([prefix]) => ({
    prefix,
    start: loop.indexOf(`this->add("${prefix}" + axis.name, coFloats)`),
  }));
  for (const [index, [prefix, member]] of groups.entries()) {
    const registration = `this->add("${prefix}" + axis.name, coFloats)`;
    const defaultValue = `ConfigOptionFloats(axis.${member})`;
    const start = registrations[index].start;
    const end = registrations[index + 1]?.start ?? loop.length;
    if (start < 0 || end < start) throw new Error(`missing ordered axis registration ${prefix}`);
    const block = loop.slice(start, end);
    if (!block.includes(registration) || !block.includes(defaultValue)) {
      throw new Error(`missing axis binding ${prefix}/${member}`);
    }
    for (const [, otherMember] of groups) {
      if (otherMember !== member && block.includes(`ConfigOptionFloats(axis.${otherMember})`)) {
        throw new Error(`crossed axis binding ${prefix}/${otherMember}`);
      }
    }
  }

  const definitions = new Map();
  for (const match of table[1].matchAll(/\{\s*"([xyze])"\s*,\s*\{([^}]+)\}\s*,\s*\{([^}]+)\}\s*,\s*\{([^}]+)\}\s*\}/g)) {
    for (const [index, member] of members.entries()) {
      const [prefix] = groups.find(([, candidate]) => candidate === member);
      definitions.set(`${prefix}${match[1]}`, {
        type: "coFloats",
        defaultValue: numberList(match[index + 2]).join(","),
      });
    }
  }
  if (definitions.size !== 12) throw new Error(`expected 12 axis definitions, got ${definitions.size}`);
  return definitions;
}

function stripComments(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, match => match.replace(/[^\n]/g, " "))
    .replace(/\/\/.*$/gm, "");
}
