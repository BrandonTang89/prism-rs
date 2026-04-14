extern crate autotools;

use autotools::Config;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use SHA256Status::{Mismatch, Unknown};

const PACKAGE_URL: &str =
    "https://github.com/cuddorg/cudd/releases/download/3.0.0/cudd-3.0.0.tar.gz";
const PACKAGE_SHA256: &str = "5fe145041c594689e6e7cf4cd623d5f2b7c36261708be8c9a72aed72cf67acce";

const CUDD_H_DECL_ANCHOR: &str =
    "extern DdNode * Cudd_addOrAbstract(DdManager *manager, DdNode *f, DdNode *cube);\n";
const CUDD_H_DECL_INSERT: &str =
    "extern DdNode * Cudd_addOrAbstract(DdManager *manager, DdNode *f, DdNode *cube);\n\
extern DdNode * Cudd_addMaxAbstract(DdManager *manager, DdNode *f, DdNode *cube);\n\
extern DdNode * Cudd_addMinAbstract(DdManager *manager, DdNode *f, DdNode *cube);\n";

const CUDD_INT_H_DECL_ANCHOR: &str =
    "extern DdNode * cuddAddOrAbstractRecur(DdManager *manager, DdNode *f, DdNode *cube);\n";
const CUDD_INT_H_DECL_INSERT: &str =
    "extern DdNode * cuddAddOrAbstractRecur(DdManager *manager, DdNode *f, DdNode *cube);\n\
extern DdNode * cuddAddMaxAbstractRecur(DdManager *manager, DdNode *f, DdNode *cube);\n\
extern DdNode * cuddAddMinAbstractRecur(DdManager *manager, DdNode *f, DdNode *cube);\n";

const CUDD_ADD_ABS_EXPORT_INSERT_ANCHOR: &str = "} /* end of Cudd_addOrAbstract */\n\n\n/*---------------------------------------------------------------------------*/\n/* Definition of internal functions                                          */\n/*---------------------------------------------------------------------------*/\n";

const CUDD_ADD_ABS_RECUR_INSERT_ANCHOR: &str = "} /* end of cuddAddOrAbstractRecur */\n\n\n\n/*---------------------------------------------------------------------------*/\n/* Definition of static functions                                            */\n/*---------------------------------------------------------------------------*/\n";

const MAX_EXPORTED_FUNCTION: &str = r#"/**
  @brief Maximization-abstracts all the variables in cube from %ADD f.

  @details Abstracts all the variables in cube from f by taking the
  maximum over all values taken by the abstracted variables.

  @return the abstracted %ADD if successful; NULL otherwise.

  @sideeffect None

  @see Cudd_addExistAbstract Cudd_addUnivAbstract Cudd_addOrAbstract

*/
DdNode *
Cudd_addMaxAbstract(
  DdManager * manager,
  DdNode * f,
  DdNode * cube)
{
    DdNode *res;

    if (addCheckPositiveCube(manager, cube) == 0) {
        (void) fprintf(manager->err,"Error: Can only abstract cubes");
        return(NULL);
    }

    do {
        manager->reordered = 0;
        res = cuddAddMaxAbstractRecur(manager, f, cube);
    } while (manager->reordered == 1);
    if (manager->errorCode == CUDD_TIMEOUT_EXPIRED && manager->timeoutHandler) {
        manager->timeoutHandler(manager, manager->tohArg);
    }

    return(res);

} /* end of Cudd_addMaxAbstract */
"#;

const MIN_EXPORTED_FUNCTION: &str = r#"/**
  @brief Minimization-abstracts all the variables in cube from %ADD f.

  @details Abstracts all the variables in cube from f by taking the
  minimum over all values taken by the abstracted variables.

  @return the abstracted %ADD if successful; NULL otherwise.

  @sideeffect None

  @see Cudd_addExistAbstract Cudd_addUnivAbstract Cudd_addOrAbstract

*/
DdNode *
Cudd_addMinAbstract(
  DdManager * manager,
  DdNode * f,
  DdNode * cube)
{
    DdNode *res;

    if (addCheckPositiveCube(manager, cube) == 0) {
        (void) fprintf(manager->err,"Error: Can only abstract cubes");
        return(NULL);
    }

    do {
        manager->reordered = 0;
        res = cuddAddMinAbstractRecur(manager, f, cube);
    } while (manager->reordered == 1);
    if (manager->errorCode == CUDD_TIMEOUT_EXPIRED && manager->timeoutHandler) {
        manager->timeoutHandler(manager, manager->tohArg);
    }

    return(res);

} /* end of Cudd_addMinAbstract */
"#;

const MAX_RECURSIVE_FUNCTION: &str = r#"/**
  @brief Performs the recursive step of Cudd_addMaxAbstract.

  @return the %ADD obtained by abstracting the variables of cube from
  f with maximization, if successful; NULL otherwise.

  @sideeffect None

*/
DdNode *
cuddAddMaxAbstractRecur(
  DdManager * manager,
  DdNode * f,
  DdNode * cube)
{
    DdNode *T, *E, *res, *res1, *res2, *one;

    statLine(manager);
    one = DD_ONE(manager);

    /* Cube is guaranteed to be a cube at this point. */
    if (cuddIsConstant(f) || cube == one) {
        return(f);
    }

    /* Abstract a variable that does not appear in f. */
    if (cuddI(manager,f->index) > cuddI(manager,cube->index)) {
        return(cuddAddMaxAbstractRecur(manager, f, cuddT(cube)));
    }

    if ((res = cuddCacheLookup2(manager, Cudd_addMaxAbstract, f, cube)) != NULL) {
        return(res);
    }

    checkWhetherToGiveUp(manager);

    T = cuddT(f);
    E = cuddE(f);

    /* If the two indices are the same, so are their levels. */
    if (f->index == cube->index) {
        res1 = cuddAddMaxAbstractRecur(manager, T, cuddT(cube));
        if (res1 == NULL) return(NULL);
        cuddRef(res1);
        res2 = cuddAddMaxAbstractRecur(manager, E, cuddT(cube));
        if (res2 == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            return(NULL);
        }
        cuddRef(res2);
        res = cuddAddApplyRecur(manager, Cudd_addMaximum, res1, res2);
        if (res == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            Cudd_RecursiveDeref(manager,res2);
            return(NULL);
        }
        cuddRef(res);
        Cudd_RecursiveDeref(manager,res1);
        Cudd_RecursiveDeref(manager,res2);
        cuddCacheInsert2(manager, Cudd_addMaxAbstract, f, cube, res);
        cuddDeref(res);
        return(res);
    } else { /* if (cuddI(manager,f->index) < cuddI(manager,cube->index)) */
        res1 = cuddAddMaxAbstractRecur(manager, T, cube);
        if (res1 == NULL) return(NULL);
        cuddRef(res1);
        res2 = cuddAddMaxAbstractRecur(manager, E, cube);
        if (res2 == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            return(NULL);
        }
        cuddRef(res2);
        res = (res1 == res2) ? res1 :
            cuddUniqueInter(manager, (int) f->index, res1, res2);
        if (res == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            Cudd_RecursiveDeref(manager,res2);
            return(NULL);
        }
        cuddDeref(res1);
        cuddDeref(res2);
        cuddCacheInsert2(manager, Cudd_addMaxAbstract, f, cube, res);
        return(res);
    }

} /* end of cuddAddMaxAbstractRecur */
"#;

const MIN_RECURSIVE_FUNCTION: &str = r#"/**
  @brief Performs the recursive step of Cudd_addMinAbstract.

  @return the %ADD obtained by abstracting the variables of cube from
  f with minimization, if successful; NULL otherwise.

  @sideeffect None

*/
DdNode *
cuddAddMinAbstractRecur(
  DdManager * manager,
  DdNode * f,
  DdNode * cube)
{
    DdNode *T, *E, *res, *res1, *res2, *one;

    statLine(manager);
    one = DD_ONE(manager);

    /* Cube is guaranteed to be a cube at this point. */
    if (cuddIsConstant(f) || cube == one) {
        return(f);
    }

    /* Abstract a variable that does not appear in f. */
    if (cuddI(manager,f->index) > cuddI(manager,cube->index)) {
        return(cuddAddMinAbstractRecur(manager, f, cuddT(cube)));
    }

    if ((res = cuddCacheLookup2(manager, Cudd_addMinAbstract, f, cube)) != NULL) {
        return(res);
    }

    checkWhetherToGiveUp(manager);

    T = cuddT(f);
    E = cuddE(f);

    /* If the two indices are the same, so are their levels. */
    if (f->index == cube->index) {
        res1 = cuddAddMinAbstractRecur(manager, T, cuddT(cube));
        if (res1 == NULL) return(NULL);
        cuddRef(res1);
        res2 = cuddAddMinAbstractRecur(manager, E, cuddT(cube));
        if (res2 == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            return(NULL);
        }
        cuddRef(res2);
        res = cuddAddApplyRecur(manager, Cudd_addMinimum, res1, res2);
        if (res == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            Cudd_RecursiveDeref(manager,res2);
            return(NULL);
        }
        cuddRef(res);
        Cudd_RecursiveDeref(manager,res1);
        Cudd_RecursiveDeref(manager,res2);
        cuddCacheInsert2(manager, Cudd_addMinAbstract, f, cube, res);
        cuddDeref(res);
        return(res);
    } else { /* if (cuddI(manager,f->index) < cuddI(manager,cube->index)) */
        res1 = cuddAddMinAbstractRecur(manager, T, cube);
        if (res1 == NULL) return(NULL);
        cuddRef(res1);
        res2 = cuddAddMinAbstractRecur(manager, E, cube);
        if (res2 == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            return(NULL);
        }
        cuddRef(res2);
        res = (res1 == res2) ? res1 :
            cuddUniqueInter(manager, (int) f->index, res1, res2);
        if (res == NULL) {
            Cudd_RecursiveDeref(manager,res1);
            Cudd_RecursiveDeref(manager,res2);
            return(NULL);
        }
        cuddDeref(res1);
        cuddDeref(res2);
        cuddCacheInsert2(manager, Cudd_addMinAbstract, f, cube, res);
        return(res);
    }

} /* end of cuddAddMinAbstractRecur */
"#;

#[derive(Debug)]
#[allow(dead_code)]
enum FetchError {
    CommandError(std::process::ExitStatus),
    IOError(std::io::Error),
    PathExists,
}

enum SHA256Status {
    Match,
    Mismatch,
    Unknown,
}

impl From<std::io::Error> for FetchError {
    fn from(err: std::io::Error) -> FetchError {
        FetchError::IOError(err)
    }
}

fn run_command(cmd: &mut Command) -> Result<(String, String), FetchError> {
    let output = cmd.output()?;

    if output.status.success() {
        Ok((
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ))
    } else {
        eprintln!("Command {:?} exited with status {}", cmd, output.status);
        Err(FetchError::CommandError(output.status))
    }
}

fn replace_once(contents: &str, from: &str, to: &str, context: &str) -> Result<String, String> {
    if !contents.contains(from) {
        return Err(format!("Could not find patch anchor for {context}."));
    }
    Ok(contents.replacen(from, to, 1))
}

fn ensure_contains_or_insert_once(
    contents: String,
    present_marker: &str,
    anchor: &str,
    insert_text: &str,
    context: &str,
) -> Result<String, String> {
    if contents.contains(present_marker) {
        return Ok(contents);
    }
    replace_once(&contents, anchor, insert_text, context)
}

fn patch_file(
    path: &Path,
    patcher: impl FnOnce(String) -> Result<String, String>,
) -> Result<(), String> {
    let original =
        fs::read_to_string(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    let patched = patcher(original)?;
    if patched
        != fs::read_to_string(path)
            .map_err(|e| format!("Cannot re-read {}: {e}", path.display()))?
    {
        fs::write(path, patched).map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
    }
    Ok(())
}

fn patch_cudd_headers(cudd_path: &Path) -> Result<(), String> {
    let cudd_h_path = cudd_path.join("cudd/cudd.h");
    let cudd_int_h_path = cudd_path.join("cudd/cuddInt.h");

    patch_file(&cudd_h_path, |contents| {
        ensure_contains_or_insert_once(
            contents,
            "Cudd_addMinAbstract",
            CUDD_H_DECL_ANCHOR,
            CUDD_H_DECL_INSERT,
            "Cudd_add{Max,Min}Abstract declarations in cudd.h",
        )
    })?;

    patch_file(&cudd_int_h_path, |contents| {
        ensure_contains_or_insert_once(
            contents,
            "cuddAddMinAbstractRecur",
            CUDD_INT_H_DECL_ANCHOR,
            CUDD_INT_H_DECL_INSERT,
            "cuddAdd{Max,Min}AbstractRecur declarations in cuddInt.h",
        )
    })
}

fn patch_cudd_add_abs(cudd_path: &Path) -> Result<(), String> {
    let cudd_add_abs_path = cudd_path.join("cudd/cuddAddAbs.c");
    patch_file(&cudd_add_abs_path, |mut contents| {
        if !contents.contains("Cudd_addMaxAbstract(") {
            let insert = format!(
                "}} /* end of Cudd_addOrAbstract */\n\n\n{}\n{}\n\n/*---------------------------------------------------------------------------*/\n/* Definition of internal functions                                          */\n/*---------------------------------------------------------------------------*/\n",
                MAX_EXPORTED_FUNCTION, MIN_EXPORTED_FUNCTION
            );
            contents = replace_once(
                &contents,
                CUDD_ADD_ABS_EXPORT_INSERT_ANCHOR,
                &insert,
                "add max/min exported functions in cuddAddAbs.c",
            )?;
        } else if !contents.contains("Cudd_addMinAbstract(") {
            let with_min = format!("{}\n{}", MAX_EXPORTED_FUNCTION, MIN_EXPORTED_FUNCTION);
            contents = replace_once(
                &contents,
                MAX_EXPORTED_FUNCTION,
                &with_min,
                "add min exported function next to max",
            )?;
        }

        if !contents.contains("DdNode *\ncuddAddMaxAbstractRecur(") {
            let insert = format!(
                "}} /* end of cuddAddOrAbstractRecur */\n\n\n{}\n{}\n\n/*---------------------------------------------------------------------------*/\n/* Definition of static functions                                            */\n/*---------------------------------------------------------------------------*/\n",
                MAX_RECURSIVE_FUNCTION, MIN_RECURSIVE_FUNCTION
            );
            contents = replace_once(
                &contents,
                CUDD_ADD_ABS_RECUR_INSERT_ANCHOR,
                &insert,
                "add max/min recursive functions in cuddAddAbs.c",
            )?;
        } else if !contents.contains("DdNode *\ncuddAddMinAbstractRecur(") {
            let with_min = format!("{}\n{}", MAX_RECURSIVE_FUNCTION, MIN_RECURSIVE_FUNCTION);
            contents = replace_once(
                &contents,
                MAX_RECURSIVE_FUNCTION,
                &with_min,
                "add min recursive function next to max",
            )?;
        }

        Ok(contents)
    })
}

fn patch_cudd_sources(cudd_path: &Path) -> Result<(), String> {
    patch_cudd_headers(cudd_path)?;
    patch_cudd_add_abs(cudd_path)
}

fn fetch_package(
    out_dir: &str,
    url: &str,
    sha256: &str,
) -> Result<(PathBuf, SHA256Status), FetchError> {
    let out_path = Path::new(out_dir);
    let target_path = out_path.join(Path::new(url).file_name().unwrap());
    let target_path_str = target_path.clone().into_os_string().into_string().unwrap();

    match target_path.metadata() {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            println!("Downloading {} to {}", url, target_path_str);
            let mut command = Command::new("curl");
            command.args(["-L", url, "-o", target_path_str.as_str()]);
            run_command(&mut command)?;
        }
        Ok(data) if data.is_file() => {
            println!("{} exists. Skipping download.", target_path_str);
        }
        Ok(_) => return Err(FetchError::PathExists),
        Err(error) => return Err(FetchError::from(error)),
    }

    let mut command_1 = Command::new("sha256sum");
    command_1.arg(target_path.clone());
    let mut command_2 = Command::new("shasum -a 256");
    command_2.arg(target_path.clone());
    let sha256_result = run_command(&mut command_1).or_else(|_| run_command(&mut command_2));

    let sha256_status = match sha256_result {
        Err(_) => SHA256Status::Unknown,
        Ok((output, _)) if output.contains(sha256) => SHA256Status::Match,
        _ => SHA256Status::Mismatch,
    };

    Ok((target_path, sha256_status))
}

fn main() -> Result<(), String> {
    let build_cudd = env::var_os("CARGO_FEATURE_BUILD_CUDD").is_some();
    if !build_cudd {
        return Ok(());
    }

    let out_dir = env::var("OUT_DIR")
        .map_err(|_| "Environmental variable `OUT_DIR` not defined.".to_string())?;

    let (tar_path, sha256_status) = fetch_package(&out_dir, PACKAGE_URL, PACKAGE_SHA256)
        .map_err(|e| format!("Error downloading CUDD package: {:?}.", e))?;
    let tar_path_str = tar_path.to_str().unwrap().to_string();

    match sha256_status {
        Unknown => eprintln!("WARNING: SHA256 not computed. Package validation skipped."),
        Mismatch => return Err("CUDD package SHA256 hash mismatch.".to_string()),
        _ => (),
    }

    let cudd_path = tar_path.with_extension("").with_extension("");
    let cudd_path_str = cudd_path.clone().into_os_string().into_string().unwrap();

    if !cudd_path.exists() {
        fs::create_dir_all(cudd_path.clone())
            .map_err(|e| format!("Cannot create CUDD directory: {:?}", e))?;
    }

    let mut tar_command = Command::new("tar");
    tar_command.args([
        "xf",
        &tar_path_str,
        "--strip-components=1",
        "-C",
        &cudd_path_str,
    ]);
    run_command(&mut tar_command).map_err(|e| format!("Error decompressing CUDD: {:?}", e))?;

    patch_cudd_sources(cudd_path.as_path())?;

    let build_output = Config::new(cudd_path).enable("dddmp", None).build();

    println!(
        "cargo:rustc-link-search=native={}",
        build_output.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=cudd");

    Ok(())
}
