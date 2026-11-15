//! Straight sketch-driven rails. Every construction reference comes from the path or
//! the section sketch; no machining feature picks a generated body face or edge.
use crate::rail::{Params, MAX_LENGTH_MM, MAX_MODELED_TAPS};
use extrusions::{
    machining::{self, FACES, TAP_DEPTH, TAP_THREAD},
    Profile,
};
use plugin_kit::{
    host::{free_body_name, sketch_id},
    input::{self, FieldError},
    name, timeline, Runner,
};
use serde_json::{json, Value};

const MAX_RAILS: usize = 16;
// SolidCraft's Wasm host allows 256 calls, including queries and guest logging.
const MAX_HOST_CALLS: usize = 256;

#[derive(Default)]
struct CallCount(usize);
impl Runner for CallCount {
    fn run(&mut self, _: &str, _: Value) -> Result<Value, String> {
        self.0 += 1;
        Ok(Value::Null)
    }
}

/// Count the actual catalog drawing commands, plus the fixed operations in `build`.
/// Reserve eight calls for selection, queries, and the guest's status/error logging.
fn call_budget(p: &Params, profile: &Profile, rails: usize) -> Result<usize, String> {
    let mut drawing = CallCount::default();
    crate::draw_profile(&mut drawing, profile)?;
    let mut per_rail = 6; // start plane, section, finish, sweep, visibility, group
    let mut sides = 0;
    for face in FACES {
        let positions = p.holes.iter().filter(|(f, _)| *f == face).count();
        if positions > 0 {
            sides += 1;
            per_rail += 8 + 6 * positions * machining::cross_holes(&p.slot, &p.grid, face).len();
        }
    }
    if sides > 0 {
        per_rail += sides + 4; // helper sketch, points, one fix, finish, visibility
    }
    for (enabled, end) in [(p.taps.start(), false), (p.taps.end(), true)] {
        if enabled {
            per_rail += p.grid.centres().len() + 5 + usize::from(end);
        }
    }
    Ok(8 + drawing.0 + 2 + 3 * rails.saturating_sub(1) + rails * per_rail)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub sketch: Option<String>,
    pub lines: Vec<String>,
    pub holes_from_end: bool,
}

pub fn parse(v: &Value) -> Result<Option<Input>, FieldError> {
    match input::choice(v, "mode", "Fixed length")? {
        "Fixed length" => return Ok(None),
        "Sketch driven" => {}
        _ => {
            return Err(FieldError::new(
                "mode",
                "must be Fixed length or Sketch driven",
            ))
        }
    }
    if input::optional_text(v, "sketch_point", "must be text")?.is_some() {
        return Err(FieldError::new(
            "sketch_point",
            "use Path lines for sketch-driven rails",
        ));
    }
    let sketch = input::optional_text(v, "sketch", "must be a sketch name")?.map(str::to_string);
    let lines = input::optional_text(v, "path_lines", "must be comma-separated line ids")?
        .unwrap_or("")
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if lines.len() > MAX_RAILS
        || lines
            .iter()
            .enumerate()
            .any(|(i, l)| lines[..i].contains(l))
    {
        return Err(FieldError::new(
            "path_lines",
            format!("give at most {MAX_RAILS} distinct line ids"),
        ));
    }
    if sketch.is_none() && !lines.is_empty() {
        return Err(FieldError::new(
            "sketch",
            "name the sketch containing Path lines",
        ));
    }
    let holes_from_end = match input::choice(v, "holes_from", "Start")? {
        "Start" => false,
        "End" => true,
        _ => return Err(FieldError::new("holes_from", "must be Start or End")),
    };
    Ok(Some(Input {
        sketch,
        lines,
        holes_from_end,
    }))
}

struct Path {
    sketch: Value,
    line: String,
    length: f64,
}

fn paths<R: Runner>(r: &mut R, input: &Input) -> Result<Vec<Path>, String> {
    let (sketch, mut ids) = if let Some(sketch) = &input.sketch {
        (json!(sketch), input.lines.clone())
    } else {
        let selection = r.run("selection.get", json!({"detail":true}))?;
        let items = selection["items"]
            .as_array()
            .ok_or("path_lines: could not read selection")?;
        let mut sketch = None;
        let mut ids = Vec::new();
        for item in items {
            if item["entity"] != "line" {
                return Err("path_lines: select straight sketch lines only".into());
            }
            let s = item["sketch"]
                .as_u64()
                .ok_or("path_lines: select sketch lines")?;
            if sketch.is_some_and(|old| old != s) {
                return Err("sketch: select lines from one sketch".into());
            }
            sketch = Some(s);
            let id = item["id"]
                .as_str()
                .ok_or("path_lines: selected line has no id")?
                .to_string();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        (
            json!(sketch.ok_or("sketch: name a sketch or select its lines")?),
            ids,
        )
    };
    let info = r.run("sketch.inspect", json!({"sketch":sketch}))?;
    let curves = info["curves"]
        .as_array()
        .ok_or("sketch: could not read curves")?;
    if ids.is_empty() {
        ids = curves
            .iter()
            .filter(|c| {
                c["type"] == "line"
                    && c["construction"] != true
                    && c["centerline"] != true
                    && c["link"].is_null()
            })
            .filter_map(|c| c["id"].as_str().map(str::to_string))
            .collect();
    }
    if ids.is_empty() || ids.len() > MAX_RAILS {
        return Err(format!(
            "path_lines: choose 1 to {MAX_RAILS} straight 2D sketch lines"
        ));
    }
    let sketch_id = info
        .get("id")
        .filter(|id| id.is_number())
        .cloned()
        .ok_or("sketch: missing sketch id")?;
    ids.into_iter()
        .map(|line| {
            let curve = curves
                .iter()
                .find(|c| c["id"] == line)
                .ok_or_else(|| format!("path_lines: no line {line}"))?;
            if curve["type"] != "line"
                || curve["construction"] == true
                || curve["centerline"] == true
                || !curve["link"].is_null()
            {
                return Err(format!(
                    "path_lines: {line} must be a drawn, non-construction straight line"
                ));
            }
            let read = |key: &str| -> Result<[f64; 2], String> {
                let a = curve[key]
                    .as_array()
                    .filter(|a| a.len() == 2)
                    .ok_or("path_lines: expected a 2D line")?;
                Ok([
                    a[0].as_f64().ok_or("path_lines: invalid coordinate")?,
                    a[1].as_f64().ok_or("path_lines: invalid coordinate")?,
                ])
            };
            let a = read("start_at")?;
            let b = read("end_at")?;
            let length = (b[0] - a[0]).hypot(b[1] - a[1]);
            if !length.is_finite() || !(0.01..=MAX_LENGTH_MM).contains(&length) {
                return Err(format!(
                    "path_lines: {line} must be 0.01 to {MAX_LENGTH_MM} mm long"
                ));
            }
            Ok(Path {
                sketch: sketch_id.clone(),
                line,
                length,
            })
        })
        .collect()
}

pub fn make<R: Runner>(r: &mut R, p: &Params, input: &Input) -> Result<Value, String> {
    let paths = paths(r, input)?;
    let tap_count = p.grid.centres().len()
        * (usize::from(p.taps.start()) + usize::from(p.taps.end()))
        * paths.len();
    if p.modeled && tap_count > MAX_MODELED_TAPS {
        return Err(format!("thread_modeled: {tap_count} taps exceed the {MAX_MODELED_TAPS} modeled taps per run; use cosmetic threads or fewer rails"));
    }
    for path in &paths {
        if path.length
            < TAP_DEPTH * (usize::from(p.taps.start()) + usize::from(p.taps.end())) as f64
        {
            return Err(format!(
                "end_taps: line {} is too short for the requested taps",
                path.line
            ));
        }
        if p.holes
            .iter()
            .any(|(_, at)| *at > path.length - machining::hole_margin())
        {
            return Err(format!(
                "holes: a hole is too close to the end of line {}",
                path.line
            ));
        }
    }
    let profile = Profile::new(p.slot, p.grid).ok_or("size: missing section")?;
    let calls = call_budget(p, &profile, paths.len())?;
    if calls > MAX_HOST_CALLS {
        return Err(format!("path_lines: this batch needs about {calls} host calls; SolidCraft allows {MAX_HOST_CALLS}. Choose fewer lines or screw holes and run smaller batches"));
    }
    let mut taken = r.body_names();
    let mut template = None;
    let mut bodies = Vec::new();
    for path in &paths {
        let body = free_body_name(
            &mut taken,
            &format!("{} {} path {}", p.slot.name, p.size, path.line),
        );
        let section = timeline::folded(r, &body, |r| {
            build(r, p, input, &profile, path, &body, template.as_ref())
        })?;
        template.get_or_insert(section);
        bodies.push(body);
    }
    Ok(
        json!({"mode":"Sketch driven","count":bodies.len(),"bodies":bodies,"lines":paths.iter().map(|p|&p.line).collect::<Vec<_>>(),"area_mm2":profile.area()}),
    )
}

fn freeze<R: Runner>(r: &mut R, sketch: &Value) -> Result<(), String> {
    let inspected = r.run("sketch.inspect", json!({"sketch":sketch}))?;
    let curves = inspected["curves"]
        .as_array()
        .ok_or("sketch.inspect: missing curves")?;
    let ids = curves
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect::<Vec<_>>();
    if !ids.is_empty() {
        r.run(
            "sketch.constraint.fix",
            json!({"entities":ids,"fixed":true}),
        )?;
    }
    Ok(())
}

fn build<R: Runner>(
    mut r: R,
    p: &Params,
    input: &Input,
    profile: &Profile,
    path: &Path,
    body: &str,
    template: Option<&Value>,
) -> Result<Value, String> {
    let path_ref = json!({"sketch":path.sketch,"curve":path.line});
    let start = name::part(body, "Start plane");
    r.run(
        "construct.plane.along_path",
        json!({"path":path_ref,"t":0,"name":start}),
    )?;
    let section = sketch_id(&r.run(
        "sketch.create",
        json!({"plane":start,"name":name::part(body,"Section")}),
    )?)?;
    if let Some(template) = template {
        // An independent copy of the geometry, without the source plane.
        r.run("sketch.replace", json!({"from":template}))?;
    } else {
        crate::draw_profile(&mut r, profile)?;
    }
    // The installed host does not retain fixes when replacing sketch contents.
    freeze(&mut r, &section)?;
    r.run("sketch.finish", json!({}))?;
    r.run("solid.sweep",json!({"sketch":section,"profiles":[{"point":profile.material_point()}],"path_sketch":path.sketch,"path":[path.line],"operation":"new","body_name":body,"name":name::part(body,"Rail")}))?;
    let sides = if p.holes.is_empty() {
        Value::Null
    } else {
        let sketch = sketch_id(&r.run(
            "sketch.create",
            json!({"plane":start,"name":name::part(body,"Side references")}),
        )?)?;
        let mut points = Vec::new();
        for (i, face) in FACES.iter().enumerate() {
            if p.holes.iter().any(|(f, _)| f == face) {
                let out = face.outward();
                let id = format!("side{i}");
                r.run("sketch.point", json!({"id":id,"point":[out[1],-out[0]]}))?;
                points.push(id);
            }
        }
        r.run(
            "sketch.constraint.fix",
            json!({"entities":points,"fixed":true}),
        )?;
        r.run("sketch.finish", json!({}))?;
        r.run(
            "browser.visibility",
            json!({"items":[sketch],"visible":false}),
        )?;
        sketch
    };
    for (i, face) in FACES.iter().enumerate() {
        let positions = p
            .holes
            .iter()
            .filter(|(f, _)| f == face)
            .map(|(_, at)| *at)
            .collect::<Vec<_>>();
        if positions.is_empty() {
            continue;
        }
        let plane = name::part(body, &format!("{} plane", face.name()));
        r.run("construct.plane.axis_point",json!({"axis":path_ref,"point":{"sketch":sides,"point":format!("side{i}")},"name":plane}))?;
        let sketch=sketch_id(&r.run("sketch.create",json!({"plane":plane,"offset":face.half(&p.grid),"name":name::part(body,&format!("Hole references {}",face.name()))}))?)?;
        let projection = r.run(
            "sketch.project",
            json!({"refs":[path_ref],"projection_link":true,"construction":true}),
        )?;
        let line = projection["created"]["curves"][0]
            .as_str()
            .ok_or("sketch.project: missing path line")?;
        let info = r.run("sketch.inspect", json!({"sketch":sketch}))?;
        let line_info = info["curves"]
            .as_array()
            .and_then(|c| c.iter().find(|c| c["id"] == line))
            .ok_or("sketch.inspect: missing projected line")?;
        let end = line_info[if input.holes_from_end { "end" } else { "start" }]
            .as_str()
            .ok_or("sketch.inspect: missing path endpoint")?;
        let holes = machining::cross_holes(&p.slot, &p.grid, *face);
        let outward = face.outward();
        let across = [outward[1], -outward[0]];
        let mut clearance = Vec::new();
        let mut access = Vec::new();
        for at in &positions {
            for hole in &holes {
                let id = format!("hole{}", clearance.len() + 1);
                let aid = format!("access{}", clearance.len() + 1);
                let y = hole.at[0] * across[0] + hole.at[1] * across[1];
                let x = if input.holes_from_end {
                    path.length - at
                } else {
                    *at
                };
                r.run(
                    "sketch.circle.center",
                    json!({"id":id,"center":[x,y],"diameter":hole.clearance.0}),
                )?;
                let center = format!("{id}.center");
                r.run(
                    "sketch.dimension",
                    json!({"entities":[end,center],"type":"horizontal","value":at}),
                )?;
                if y.abs() < 1e-9 {
                    r.run("sketch.constraint.coincident", json!({"a":center,"b":line}))?;
                } else {
                    r.run("sketch.dimension",json!({"entities":[line_info["start"],center],"type":"vertical","value":y.abs()}))?;
                }
                for (circle, diameter) in [(&id, hole.clearance.0), (&aid, hole.access.0)] {
                    if circle == &aid {
                        r.run(
                            "sketch.circle.center",
                            json!({"id":aid,"center":center,"diameter":diameter}),
                        )?;
                    }
                    r.run(
                        "sketch.dimension",
                        json!({"entities":[circle],"type":"diameter","value":diameter}),
                    )?;
                }
                clearance.push(json!({"curves":[id]}));
                access.push(json!({"curves":[aid]}));
            }
        }
        r.run("sketch.finish", json!({}))?;
        for (label, profiles, depth) in [
            ("Access", access, holes[0].access.1),
            ("Clearance", clearance, holes[0].clearance.2),
        ] {
            r.run("solid.extrude",json!({"sketch":sketch,"profiles":profiles,"distance":depth,"direction":"negative","operation":"cut","targets":[body],"name":name::part(body,&format!("{} {label}",face.name()))}))?;
        }
        r.run(
            "browser.visibility",
            json!({"items":[sketch],"visible":false}),
        )?;
    }
    for (enabled, end) in [(p.taps.start(), false), (p.taps.end(), true)] {
        if !enabled {
            continue;
        }
        let label = if end { "End" } else { "Start" };
        let plane = if end {
            let plane = name::part(body, "End plane");
            r.run(
                "construct.plane.along_path",
                json!({"path":path_ref,"t":1,"name":plane}),
            )?;
            plane
        } else {
            start.clone()
        };
        let sketch = sketch_id(&r.run(
            "sketch.create",
            json!({"plane":plane,"name":name::part(body,&format!("{label} taps"))}),
        )?)?;
        let mut points = Vec::new();
        for (i, at) in p.grid.centres().iter().enumerate() {
            let id = format!("tap{i}");
            r.run("sketch.point", json!({"id":id,"point":at}))?;
            points.push(id);
        }
        r.run(
            "sketch.constraint.fix",
            json!({"entities":points,"fixed":true}),
        )?;
        r.run("sketch.finish", json!({}))?;
        r.run("solid.hole",json!({"sketch":sketch,"points":points,"flip":!end,"thread":TAP_THREAD,"depth":TAP_DEPTH,"thread_depth":TAP_DEPTH,"modeled":p.modeled,"targets":[body],"name":name::part(body,&format!("{label} tap"))}))?;
        r.run(
            "browser.visibility",
            json!({"items":[sketch],"visible":false}),
        )?;
    }
    r.run(
        "browser.visibility",
        json!({"items":[section],"visible":false}),
    )?;
    Ok(section)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_keeps_fixed_length_default_and_rejects_ambiguous_paths() {
        assert_eq!(parse(&json!({})).unwrap(), None);
        for (value, field) in [
            (json!({"mode":"other"}), "mode"),
            (json!({"mode":"Sketch driven","path_lines":"l1"}), "sketch"),
            (
                json!({"mode":"Sketch driven","sketch":"Path","path_lines":"l1,l1"}),
                "path_lines",
            ),
            (
                json!({"mode":"Sketch driven","sketch_point":"p1"}),
                "sketch_point",
            ),
            (
                json!({"mode":"Sketch driven","holes_from":"Middle"}),
                "holes_from",
            ),
        ] {
            assert_eq!(parse(&value).unwrap_err().field, field);
        }
    }

    #[test]
    fn path_mode_uses_shared_profile_validation_but_not_fixed_length() {
        let p=crate::rail::parse(&json!({"mode":"Sketch driven","sketch":"Path","slot":"V-slot","size":"2080","length":0,"holes_from":"End"})).unwrap();
        assert_eq!(p.grid.ny, 4);
        assert!(p.path.unwrap().holes_from_end);
        assert_eq!(
            crate::rail::parse(&json!({"mode":"Sketch driven","slot":"V-slot","size":"4040"}))
                .unwrap_err()
                .field,
            "size"
        );
    }

    struct ReadSketch {
        info: Value,
        calls: Vec<String>,
        selection: Vec<Value>,
        inspected: Vec<Value>,
    }
    impl Runner for ReadSketch {
        fn run(&mut self, command: &str, args: Value) -> Result<Value, String> {
            self.calls.push(command.into());
            match command {
                "selection.get" => Ok(json!({"items":self.selection})),
                "sketch.inspect" => {
                    self.inspected.push(args["sketch"].clone());
                    Ok(self.info.clone())
                }
                _ => Err("unexpected mutation".into()),
            }
        }
    }
    fn reader() -> ReadSketch {
        ReadSketch {
            calls: vec![],
            selection: vec![],
            inspected: vec![],
            info: json!({"id":7,"curves":[
                {"id":"l1","type":"line","start_at":[0,0],"end_at":[0,100]},
                {"id":"l2","type":"line","start_at":[0,0],"end_at":[100,0],"construction":true},
                {"id":"c1","type":"circle"}
            ]}),
        }
    }
    #[test]
    fn selected_sketches_are_resolved_and_compared_by_id_not_name() {
        let mut r = reader();
        let input = Input {
            sketch: None,
            lines: vec![],
            holes_from_end: false,
        };
        let selected =
            |sketch| json!({"entity":"line","sketch":sketch,"sketch_name":"Paths","id":"l1"});
        r.selection = vec![selected(7)];
        let found = paths(&mut r, &input).unwrap();
        assert_eq!(found[0].sketch, json!(7));
        assert_eq!(r.inspected, [json!(7)]);

        let mut r = reader();
        r.selection = vec![selected(7), selected(8)];
        assert_eq!(
            paths(&mut r, &input).err().as_deref(),
            Some("sketch: select lines from one sketch")
        );
        assert_eq!(r.calls, ["selection.get"]);
    }
    #[test]
    fn all_lines_excludes_construction_and_explicit_invalid_lines_fail_before_writes() {
        let mut r = reader();
        let input = Input {
            sketch: Some("Path".into()),
            lines: vec![],
            holes_from_end: false,
        };
        let found = paths(&mut r, &input).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].length, 100.0);
        for line in ["l2", "c1", "missing"] {
            let mut input = input.clone();
            input.lines = vec![line.into()];
            assert!(paths(&mut r, &input).is_err());
        }
        assert!(r.calls.iter().all(|c| c == "sketch.inspect"));
    }
    #[test]
    fn machining_limits_are_checked_on_actual_path_length_before_writes() {
        let mut r = reader();
        let p = crate::rail::parse(&json!({"mode":"Sketch driven","sketch":"Path","holes":"150"}))
            .unwrap();
        assert!(make(&mut r, &p, p.path.as_ref().unwrap())
            .unwrap_err()
            .starts_with("holes:"));
        r.info["curves"][0]["end_at"] = json!([0, 20]);
        let p =
            crate::rail::parse(&json!({"mode":"Sketch driven","sketch":"Path","end_taps":"Both"}))
                .unwrap();
        assert!(make(&mut r, &p, p.path.as_ref().unwrap())
            .unwrap_err()
            .starts_with("end_taps:"));
        assert!(r.calls.iter().all(|c| c == "sketch.inspect"));
    }
    #[test]
    fn budget_rejects_large_machined_batches_before_writes() {
        let mut r = reader();
        r.info["curves"].as_array_mut().unwrap().truncate(1);
        for i in 2..=16 {
            let mut line = r.info["curves"][0].clone();
            line["id"] = json!(format!("l{i}"));
            r.info["curves"].as_array_mut().unwrap().push(line);
        }
        let p = crate::rail::parse(&json!({"mode":"Sketch driven","sketch":"Path","size":"4040","end_taps":"Both","thread_modeled":false,"holes":"+X 25 -Y 40"})).unwrap();
        let error = make(&mut r, &p, p.path.as_ref().unwrap()).unwrap_err();
        assert!(error.contains("host calls"), "{error}");
        assert_eq!(r.calls, ["sketch.inspect"]);
    }

    struct BuildHost(Vec<String>);
    impl Runner for BuildHost {
        fn run(&mut self, command: &str, _: Value) -> Result<Value, String> {
            self.0.push(command.into());
            Ok(match command {
                "sketch.create" => json!({"sketch":self.0.len()}),
                "sketch.inspect" => json!({"curves":[{"id":"l1","start":"p1","end":"p2"}]}),
                "sketch.project" => json!({"created":{"curves":["l1"]}}),
                _ => json!({"feature":self.0.len()}),
            })
        }
    }

    #[test]
    fn budget_covers_real_build_commands_and_manifest_authorizes_them() {
        let manifest: Value = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        let allowed = manifest["runtime"]["commands"].as_array().unwrap();
        for slot in ["T-slot", "V-slot"] {
            for size in ["2020", "2040", "2060", "2080", "4040"] {
                if slot == "V-slot" && size == "4040" {
                    continue;
                }
                for holes in ["", "+X 25 -Y 40 +Y 60 -X 80"] {
                    let p = crate::rail::parse(&json!({"mode":"Sketch driven","sketch":"Path","slot":slot,"size":size,"end_taps":"Both","thread_modeled":false,"holes":holes})).unwrap();
                    let profile = Profile::new(p.slot, p.grid).unwrap();
                    let mut host = BuildHost(vec![]);
                    let mut template = None;
                    for i in 1..=3 {
                        let path = Path {
                            sketch: json!(1),
                            line: format!("l{i}"),
                            length: 100.0,
                        };
                        let section = timeline::folded(&mut host, "Rail", |r| {
                            build(
                                r,
                                &p,
                                p.path.as_ref().unwrap(),
                                &profile,
                                &path,
                                "Rail",
                                template.as_ref(),
                            )
                        })
                        .unwrap();
                        template.get_or_insert(section);
                    }
                    assert_eq!(call_budget(&p, &profile, 3).unwrap(), host.0.len() + 8);
                    assert!(host.0.iter().all(|c| allowed.contains(&json!(c))));
                }
            }
        }
    }
}
