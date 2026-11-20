//! `bm_gym._core`: the bm-gym environment for Python. The Gymnasium wrapper lives in
//! `python/bm_gym/env.py`; this module only moves numbers across.

use bm_engine::ManagerStyle;
use bm_gym::{action_name, EnvConfig, Info, ManagerEnv, Reward, N_ACTIONS, N_OBS, OBS_NAMES};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn style(name: &str) -> PyResult<ManagerStyle> {
    match name {
        "mlb" => Ok(ManagerStyle::mlb()),
        "mlb_2022" => Ok(ManagerStyle::mlb_2022()),
        "none" => Ok(ManagerStyle::none()),
        other => Err(PyValueError::new_err(format!("unknown manager style '{other}' (mlb, mlb_2022, none)"))),
    }
}

fn info_dict<'py>(py: Python<'py>, i: &Info) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("inning", i.inning)?;
    d.set_item("top", i.top)?;
    d.set_item("outs", i.outs)?;
    d.set_item("balls", i.balls)?;
    d.set_item("strikes", i.strikes)?;
    d.set_item("bases", i.bases)?;
    d.set_item("score", (i.score[0], i.score[1]))?;
    d.set_item("win_expectancy", i.win_expectancy)?;
    d.set_item("side", i.side)?;
    d.set_item("decisions", i.decisions)?;
    d.set_item("won", i.won)?;
    Ok(d)
}

/// What `step` returns: (observation, reward, terminated, truncated, info).
type StepTuple<'py> = (Vec<f32>, f64, bool, bool, Bound<'py, PyDict>);

/// One game at a time: the agent orders every pitch to its club's hitters.
#[pyclass(module = "bm_gym._core", name = "ManagerEnv")]
#[derive(Clone)]
struct PyManagerEnv {
    env: ManagerEnv,
}

#[pymethods]
impl PyManagerEnv {
    #[new]
    #[pyo3(signature = (seed = 0, reward = "win_expectancy", side = None, league_seed = 2025, opponent = "mlb", own = "mlb"))]
    fn new(seed: u64, reward: &str, side: Option<usize>, league_seed: u64, opponent: &str, own: &str) -> PyResult<Self> {
        let reward = match reward {
            "win_expectancy" => Reward::WinExpectancy,
            "win" => Reward::Win,
            other => return Err(PyValueError::new_err(format!("unknown reward '{other}' (win_expectancy, win)"))),
        };
        if matches!(side, Some(s) if s > 1) {
            return Err(PyValueError::new_err("side is 0 (away), 1 (home) or None (random)"));
        }
        let cfg = EnvConfig { reward, side, opponent: style(opponent)?, own: style(own)?, league_seed };
        Ok(PyManagerEnv { env: ManagerEnv::new(cfg, seed) })
    }

    /// Start a new game; returns (observation, info).
    #[pyo3(signature = (seed = None))]
    fn reset<'py>(&mut self, py: Python<'py>, seed: Option<u64>) -> PyResult<(Vec<f32>, Bound<'py, PyDict>)> {
        let (obs, info) = self.env.reset(seed);
        Ok((obs.to_vec(), info_dict(py, &info)?))
    }

    /// Order the next pitch; returns (observation, reward, terminated, truncated, info).
    fn step<'py>(&mut self, py: Python<'py>, action: usize) -> PyResult<StepTuple<'py>> {
        if action >= N_ACTIONS {
            return Err(PyValueError::new_err(format!("action {action} out of range 0..{N_ACTIONS}")));
        }
        if self.env.sim().is_none() || self.env.sim().is_some_and(|g| g.is_over()) {
            return Err(PyValueError::new_err("no game in progress: call reset()"));
        }
        let r = self.env.step(action);
        Ok((r.obs.to_vec(), r.reward, r.terminated, r.truncated, info_dict(py, &r.info)?))
    }

    /// Actions that make sense before the next pitch.
    fn action_mask(&self) -> PyResult<Vec<bool>> {
        if self.env.sim().is_none() {
            return Err(PyValueError::new_err("call reset() first"));
        }
        Ok(self.env.action_mask().to_vec())
    }

    /// A copy of the environment, game in progress included (to try an action several ways).
    fn copy(&self) -> Self {
        self.clone()
    }

    /// Reseed the environment's random numbers.
    fn seed(&mut self, seed: u64) {
        self.env.seed(seed);
    }

    /// Play the rest of the game with the club's offence handed to its AI manager; returns
    /// whether the agent's club won.
    fn finish_with_ai(&mut self) -> PyResult<bool> {
        if self.env.sim().is_none() {
            return Err(PyValueError::new_err("call reset() first"));
        }
        Ok(self.env.finish_with_ai())
    }

    /// The side the agent manages in the current game (0 = away, 1 = home).
    #[getter]
    fn side(&self) -> usize {
        self.env.side()
    }

    #[staticmethod]
    fn observation_names() -> Vec<&'static str> {
        OBS_NAMES.to_vec()
    }

    #[staticmethod]
    fn action_names() -> Vec<String> {
        (0..N_ACTIONS).map(action_name).collect()
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyManagerEnv>()?;
    m.add("N_ACTIONS", N_ACTIONS)?;
    m.add("N_OBS", N_OBS)?;
    Ok(())
}
