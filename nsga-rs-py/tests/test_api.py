"""This file contains tests related to the Python package."""

from pathlib import Path

import matplotlib.pyplot as plt
import pytest
from nsga_rs import (
    NSGA3Data,
    ObjectiveDirection,
    NSGA2Data,
    DasDarren1998,
    TwoLayerPartitions,
    StoppingConditionValue,
    StoppingCondition,
    NSGA2Arg,
    SimulatedBinaryCrossoverArgs,
    Algorithm,
    NSGA3Arg,
)


class TestPythonAPI:
    """Test class."""

    root_folder = Path(__file__).parent.parent.parent

    def test_reader(self) -> None:
        """
        Test the JSON reader.
        :return: None.
        """
        file = (
            self.root_folder / "examples" / "results" / "DTLZ1_3obj_NSGA3_gen400.json"
        )
        data = NSGA3Data(file)
        p = data.problem

        assert p.number_of_variables == 7
        assert p.variables["x1"].min_value == 0

        assert len(p.objectives) == 3
        assert p.objectives["f1"].direction == ObjectiveDirection.Minimise

        assert "g" in p.constraints.keys()
        assert data.algorithm == "NSGA3"
        assert data.generation == 400
        assert data.took.seconds == 4
        assert data.exported_on.day == 10

        assert data.individuals[0].constraint_violation() == 0
        assert (
            pytest.approx(data.individuals[0].get_objective_value("f2"), abs=1e-3)
            == 0.167
        )
        assert "x5" in data.individuals[0].variables().keys()
        assert "reference_point_index" in data.individuals[0].data().keys()

        assert pytest.approx(data.hyper_volume([100, 100, 100]), abs=1e-2) == 999999.97

    def test_plot(self) -> None:
        """
        Test the plot method.
        :return: None.
        """
        assert isinstance(
            NSGA2Data(
                self.root_folder / "examples" / "results" / "SCH_2obj_NSGA2_gen250.json"
            ).plot(),
            plt.Figure,
        )

        assert isinstance(
            NSGA3Data(
                self.root_folder
                / "examples"
                / "results"
                / "DTLZ1_3obj_NSGA3_gen400.json"
            ).plot(),
            plt.Figure,
        )

        assert isinstance(
            NSGA3Data(
                self.root_folder
                / "examples"
                / "results"
                / "DTLZ1_8obj_NSGA3_gen750.json"
            ).plot(),
            plt.Figure,
        )

        assert isinstance(
            NSGA2Data.plot_convergence(
                (self.root_folder / "examples" / "results" / "convergence").as_posix(),
                [10000, 10000],
            ),
            plt.Figure,
        )

    def test_reference_points(self) -> None:
        """
        Test the reference point methods.
        :return: None.
        """
        ds = DasDarren1998(3, 5)
        points = ds.calculate()
        assert len(points) == 21
        assert isinstance(ds.plot(points), plt.Figure)

        two_layers = TwoLayerPartitions(
            boundary_layer=3,
            inner_layer=4,
            scaling=None,
        )
        ds = DasDarren1998(3, two_layers)
        assert len(ds.calculate()) == 25

    def test_stopping_condition(self) -> None:
        """
        Test the StoppingCondition class.
        :return: None.
        """
        c1 = StoppingConditionValue.max_duration_as_minutes(3)
        assert c1.value() == 3
        assert StoppingCondition(condition=c1).conditions()[0].value() == c1.value()

        c2 = StoppingConditionValue.max_generation(300)
        cond = StoppingCondition(condition=[c1, c2])
        assert cond.conditions()[0].value() == c1.value()
        assert cond.conditions()[1].value() == c2.value()

    def test_nsga2_args(self) -> None:
        """
        Test the NSGA2Arg class.
        :return: None.
        """
        stopping_condition = StoppingCondition(
            condition=StoppingConditionValue.max_duration_as_minutes(3)
        )
        sbx = SimulatedBinaryCrossoverArgs(
            distribution_index=1, crossover_probability=0.9
        )
        args = NSGA2Arg(
            number_of_individuals=10,
            stopping_condition=stopping_condition,
            crossover_operator_options=sbx,
            parallel=True,
        )
        assert args.number_of_individuals == 10

        assert args.crossover_operator_options.distribution_index == 1
        assert args.crossover_operator_options.crossover_probability == 0.9
        assert args.crossover_operator_options.variable_probability == 0.5

        assert args.mutation_operator_options is None
        assert args.resume_from_file is None

        assert args.parallel
        assert args.export_history is None

        Algorithm.nsga2(args)

    def test_nsga3_args(self) -> None:
        """
        Test the NSGA3Arg class.
        :return: None.
        """
        stopping_condition = StoppingCondition(
            condition=StoppingConditionValue.max_duration_as_minutes(3)
        )
        args = NSGA3Arg(
            number_of_partitions=10,
            number_of_individuals=10,
            stopping_condition=stopping_condition,
            parallel=True,
        )
        assert args.number_of_individuals == 10
        assert args.number_of_partitions == 10

        assert args.crossover_operator_options is None
        assert args.mutation_operator_options is None
        assert args.resume_from_file is None

        assert args.parallel
        assert args.export_history is None

        Algorithm.nsga3(args)
