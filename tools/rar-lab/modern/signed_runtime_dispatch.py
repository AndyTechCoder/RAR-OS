"""Fixed trusted-main signed runtime dispatch; no arguments."""
import importlib.util
from pathlib import Path
path=Path(__file__).resolve().with_name("runtime_controller.py")
spec=importlib.util.spec_from_file_location("runtime_controller",path)
controller=importlib.util.module_from_spec(spec);spec.loader.exec_module(controller)
if __name__=="__main__":controller.main("signed-runtime")
