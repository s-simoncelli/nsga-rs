from datetime import timedelta, datetime
from enum import Enum
from pathlib import Path

import matplotlib.pyplot as plt

class ObjectiveDirection(Enum):
    """A class describing the objective direction."""

    Minimise = "minimise"
    """ The objective is minimised """
    Maximise = "maximise"
    """ The objective is maximised """

class Objective:
    """An objective set on the solved problem."""

    name: str
    """ The objective name. """
    direction: ObjectiveDirection
    """ Whether the objective should be minimised or maximised. """

class RelationalOperator(Enum):
    """Operator used to check a bounded constraint."""

    pass

class Constraint:
    """A constraint set on the solved problem."""

    name: str
    """ The constraint name """
    operator: RelationalOperator
    """ The relational operator that's used to compare a value against the constraint 
    target value """
    target: float
    """ The constraint target """

class Variable:
    """A variable set on the solved problem."""

    name: str
    """ The variable name """
    var_type: VariableType
    """ An enumerator class identifying the type of variable """
    min_value: float | None
    """ The minimum bound. This is None if the variable does not support bounds. """
    max_value: float | None
    """ The maximum bound. This is None if the variable does not support bounds. """

class Problem:
    """Class holding information about the solved problem."""

    objectives: dict[str, Objective]
    """ The problem objectives. The list contains classes of Objective 
    instances that describe how each objective was configured. """
    constraints: dict[str, Constraint]
    """ The problem constraints. The list contains classes of Constraint 
    instances that describe how each constraint was configured. This 
    is an empty list if no constraints were defined in the solved problem. """
    variables: dict[str, Variable]
    """ The problem variables. The list contains classes of Variable instances
    that describe the type of each variable and how this was configured."""
    constraint_names: list[str]
    """ The constraint names. """
    variable_names: list[str]
    """ The variable names. """
    objective_names: list[str]
    """ The objective names. """
    number_of_objectives: int
    """ The number of objectives """
    number_of_constraints: int
    """ The number of constraints. """
    number_of_variables: int
    """ The number of variables. """

type VariableType = float | int | bool | str
type DataType = float | int | list[DataType] | dict[str, DataType]

class Individual:
    """
    An individual in the population containing the problem solution, and the
    objective and constraint values.
    """

    def constraint_violation(self) -> float:
        """
        Get the overall amount of violation of the solution constraints. This is a
        measure about how close (or far) the individual meets the constraints. If the
        solution is feasible, then the violation is 0.0. Otherwise, a positive number
        is returned.
        :return: The violation amount.
        """
        ...

    def is_feasible(self) -> bool:
        """
        Get whether the solution meets all the problem constraints.
        :return: True is the solution is feasible.
        """
        ...

    def variables(self) -> dict[str, VariableType]:
        """
        Get the variables.
        :return: A dictionary with the variable names and values for the individual.
        """
        ...

    def objectives(self) -> dict[str, float]:
        """
        Get the objectives.
        :return: A dictionary with the objective names and values for the individual.
        """
        ...

    def constraints(self) -> dict[str, float]:
        """
        Get the constraints.
        :return: A dictionary with the constraint names and values for the individual.
        """
        ...

    def get_objective_value(self, name: str) -> float:
        """
        Get the objective value by name. This returns an error if the objective does
        not exist.
        :param name: The objective name.
        :return: The objective value.
        """

    def get_constraint_value(self, name: str) -> float:
        """
        Get the constraint value by name. This return an error if the constraint name
        does not exist.
        :param name: The constraint name.
        :return: The constraint value.
        """

    def get_variable_value(self, name: str) -> VariableType:
        """
        Get the variable value by name. This return an error if the variable name
        does not exist.
        :param name: The variable name.
        :return: The variable value. The type depends how the variable was configured
        when the problem was optimised.
        """

    def get_variable_values(self) -> [VariableType]:
        """
        Get the vector with the variable values for the individual.
        :return: All the individual's variables.
        """
        ...

    def get_objective_values(self) -> [float]:
        """
        Get the list with the objective values for the individual. The size of the
        list will equal the number of problem objectives.
        :return: All the individual's objectives.
        """
        ...

    def data(self) -> dict[str, DataType]:
        """
        Get additional numeric data stores in the individuals (such as crowding
        distance or rank) depending on the algorithm the individuals are derived from.
        :return: The dictionary with the data name as key and its value as value.
        """
        ...

class AlgorithmData:
    """Class holding the algorithm data."""

    problem: Problem
    """ The problem. This class holds information about the solved problem. """
    generation: int
    """ The generation the export was collected at """
    algorithm: str
    """ Get the algorithm name used to evolve the individuals """
    took: timedelta
    """ The time took to reach the generation """
    individuals: list[Individual]
    """ The list with the individuals. An individual in the population contains
     the problem solution, and the objective and constraint values. """
    objectives: dict[str, list[float]]
    """ The objective values grouped by objective name """
    additional_data: dict[str, DataType] | None
    """ Any additional data exported by the algorithm (such as the distance to
    the reference point for NSGA3) """
    exported_on: datetime
    """  The date and time when the parsed JSON file was exported """

    def __init__(self, file: Path):
        """
        Initialise the NSGA2 file reader.
        :param file: The path to the JSON file exported from optirustic Rust library.
        """

    def hyper_volume(self, reference_point: list[float]) -> float:
        """
        Calculate the exact hyper-volume metric. Depending on the number of problem
        objectives, a different method is used to ensure a correct and fast calculation:
         - with 2 objectives: by calculating the rectangle areas between each objective
           point and the reference point.
         - with 3 objectives: by using the algorithm proposed by Fonseca et al. (2006)
           (https://dx.doi.org/10.1109/CEC.2006.1688440).
         - with 4 or more objectives:  by using the algorithm proposed by While et al.
           (2012) (https://dx.doi.org/10.1109/TEVC.2010.2077298).
        :param reference_point: The reference or anti-optimal point to use in the
        calculation. If you are not sure about the point to use you could pick the worst
        value of each objective from the individual's values. The size of this point
        must match the number of problem objectives.
        :return: The hyper volume.
        """

    def estimate_reference_point(self, offset: list[float | None]) -> list[float]:
        """
        Calculate a reference point by taking the maximum of each objective (or
        minimum if the objective is maximised) from the calculated individual's
        objective values, so that the point will be dominated by all other points. An
        optional offset for all objectives could also be added or removed to enforce
        strict dominance (if the objective is minimised the offset is added to the
        calculated reference point, otherwise it is subtracted).
        :param offset: The offset to add to each objective coordinate of the calculated
        reference point. This must have a size equal to the number of objectives in the
        problem (`self.problem.number_of_objectives`).
        :return: The reference point. This returns an error if there are no individuals
        or the size of the offset does not match `self.problem.number_of_objectives`.
        """

    @staticmethod
    def estimate_reference_point_from_files(
        folder: str, offset: list[float | None]
    ) -> list[float]:
        """
        Calculate a reference point by taking the maximum of each objective (or
        minimum if the objective is maximised) from the objective values exported in a
        JSON files. This may be use to estimate the reference point when convergence is
        being tracked and one dominated reference point is needed.
        :param folder: The path to the folder with the JSON files.
        :param offset: The offset to add to each objective coordinate of the calculated
        reference point. This must have a size equal to the number of objectives in the
        problem (`self.problem.number_of_objectives`).
        :return: The reference point. This returns an error if there are no individuals,
        the folder does not exist or the size of the offset does not match
        `self.problem.number_of_objectives`.
        """
        ...

    def plot(self) -> plt.Figure:
        """
        Plot the Pareto front with the objective values. With 2 or 3 objectives, a 2D
        or 3D chart is rendered respectively. With multi-objective problem a parallel
        coordinate chart is generated.
        This function only generates the matplotlib chart; you can manipulate the
        figure, save it (using `self.plot().savefig("figure.png")`) or show it (using
        `plt.show()`).
        :return: The `matplotlib`  figure object.
        """
        ...

    @staticmethod
    def convergence_data(
        folder: str, reference_point: list[float]
    ) -> tuple[list[int], list[datetime], list[float]]:
        """
        Calculate the hyper-volume at different generations (using the serialised
        objective values in JSON files exported at different generations).
        :param folder: The folder with the JSON files exported by the algorithm.
        :param reference_point: The reference or anti-optimal point to use in the
        calculation. The size of this point must match the number of problem objectives
        and must be dominated by all objectives at all generations.
        :return: A tuple containing the list of generation numbers, datetime objects,
        when the JSOn files were exported, and the hyper-volume values.
        """
        ...

    @staticmethod
    def plot_convergence(folder: str, reference_point: list[float]) -> plt.Figure:
        """
        Calculate the hyper-volume at different generations (using the serialised
        objective values in JSON files exported at different generations) and shows
        a convergence chart.
        :param folder: The folder with the JSON files exported by the algorithm.
        :param reference_point: The reference or anti-optimal point to use in the
        calculation. The size of this point must match the number of problem objectives
        and must be dominated by all objectives at all generations.
        :return: The figure object.
        """

class NSGA2Data(AlgorithmData):
    """Class to parse data exported with the NSGA2 algorithm."""

    pass

class NSGA3Data(AlgorithmData):
    """Class to parse data exported with the NSGA3 algorithm."""

    def plot_reference_points(self, reference_points: list[list[float]]) -> plt.Figure:
        """
        Generate a chart showing the reference point locations used by the algorithm
        and generated with the Das & Darren (2019) method.
        :param reference_points: The reference points.
        :return: The figure object.
        """

class TwoLayerPartitions:
    """Define the number of partitions for the two layers."""

    boundary_layer: int
    """ This is the number of partitions to use in the boundary layer. """
    inner_layer: int
    """ This is the number of partitions to use in the inner layer. """
    scaling: float | None
    """ Control the size of the inner layer. This defaults to 0.5 which means that the
    maximum points on each objectives axis will be located at 0.5 instead of 1 (as in
    the boundary layer). """

    def __init__(
        self, boundary_layer: int, inner_layer: int, scaling: float | None = None
    ):
        """
        Initialise the class.
        :param boundary_layer: The number of partitions to use in the boundary layer.
        :param inner_layer: The number of partitions to use in the inner layer.
        :param scaling: Control the size of the inner layer. This defaults to 0.5 which
        means that the maximum points on each objectives axis will be located at 0.5
        instead of 1 (as in the boundary layer).
        """
        ...

class DasDarren1998:
    """
    Derive the reference points or weights using the methodology suggested in Section
    5.2 in the Das & Dennis (1998) paper (https://doi.org/10.1137/S1052623496307510).
    """

    number_of_objectives: int
    """The number of problem objectives."""
    number_of_partitions: int | TwoLayerPartitions
    """The number of uniform gaps between two consecutive points along all objective 
    axis on the hyperplane. With this option you can create one or two layer of points
    with different spacing."""

    def __init__(
        self, number_of_objectives: int, number_of_partitions: int | TwoLayerPartitions
    ):
        """
        Derive the reference points or weights using the methodology suggested by
        Das & Dennis (1998).
        :param number_of_objectives: The number of problem objectives.
        :param number_of_partitions: The number of uniform gaps between two consecutive
        points along all objective axis on the hyperplane. With this option you can
        create one or two layer of points with different spacing: To create:
          - 1 layer or set of points with a constant uniform gaps use a 'int'.
          - 2 layers of points with each layer having a different gap use a
            dictionary with the following keys: inner_layer (the number of partitions
            to use in the inner layer), boundary_layer (the number of partitions to
            use in the boundary layer) and scaling (to control the size of the inner)
            layer. This defaults to 0.5 which means that the maximum points on each
            objectives axis will be located at 0.5 instead of 1 (as in the boundary
            layer).
        Use the 2nd approach if you are trying to solve a problem with many objectives
        (4 or more) and want to reduce the number of reference points to use. Using two
        layers allows  (1) setting a smaller number of reference points, (2) controlling
        the point density in the inner area and (3) ensure a well-spaced point
        distribution.
        """
        ...

    def calculate(self) -> list[list[float]]:
        """
        Generate the vector of weights of reference points.
        :return: The vector of weights of size `number_of_points`. Each  nested list,
        of size equal to `number_of_objectives`, contains the relative coordinates
        (between 0 and 1) of the points for each objective.
        """
        ...

    @staticmethod
    def plot(reference_points: list[list[float]]) -> plt.Figure:
        """
        Generate a chart showing the reference point locations (for example using the
        Das & Darren (2019) method).
        :param reference_points: The reference points.
        :return: The figure object.
        """
        ...

class StoppingConditionValue:
    """Class to use to define the stopping condition."""

    def value(self) -> int:
        """
        Get the stopping condition value.
        :return: The value.
        """
        ...

    @classmethod
    def max_duration(cls, duration: int) -> StoppingConditionValue:
        """
        Initialise the stopping condition. This will stop the algorithm when the
        elapsed time exceeds the given duration.
        :param duration: The duration in seconds.
        :return: The StoppingConditionValue instance.
        """
        ...

    @classmethod
    def max_generation(cls, generation: int) -> StoppingConditionValue:
        """
        Initialise the stopping condition. This will stop the algorithm when the
        number of generations of the evolved population exceeds the given number.
        :param generation: The population generation.
        :return: The StoppingConditionValue instance.
        """
        ...

    @classmethod
    def max_function_evaluations(cls, nfe: int) -> StoppingConditionValue:
        """
        Initialise the stopping condition. This will stop the algorithm when the
        number of function evaluations exceeds the given number. An evaluation takes
        place every time the algorithm needs to calculate an individual's objectives and
        constraints.
        :param nfe: The maximum evaluations.
        :return: The StoppingConditionValue instance.
        """
        ...

class StoppingCondition:
    """
    Define the stopping condition. This can be one conditions or multiple
    conditions.
    """

    def __init__(
        self, condition: StoppingConditionValue | list[StoppingConditionValue]
    ):
        """
        Initialise the class.
        :param condition: The condition(s). If you provide a list, the algorithm will
        stop when at least one condition is met.
        """
        ...

    def conditions(self) -> [StoppingConditionValue]:
        """
        Get a list of set stopping conditions.
        :return: The list of StoppingConditionValue instances.
        """
        ...

class PolynomialMutationArgs:
    """The Polynomial mutation (PM) operator options."""

    index_parameter: float
    """A user-defined parameter to control the mutation."""
    variable_probability: float
    """The probability of mutating a parent variable."""

    def __init__(
        self, variable_probability: float, index_parameter: float | None = None
    ):
        """
        Initialise the Polynomial mutation (PM) operator with the default parameters.
        With a distribution index or index parameter of `20` and variable probability
        equal `1` divided by  the number of real variables in the
        problem (i.e. each variable will have the same probability of being mutated).
        :param variable_probability: The probability of mutating a parent variable.
        :param index_parameter: A user-defined parameter to control the mutation. This
        is eta_m in the paper, and it is suggested its value to be in the [20, 100]
        range.
        """
        ...

class SimulatedBinaryCrossoverArgs:
    """Inputs for the SimulatedBinaryCrossover."""

    distribution_index: float
    """The distribution index for crossover."""
    crossover_probability: float
    """The probability that the parents participate in the crossover."""
    variable_probability: float
    """The probability that a variable belonging to both parents is used in the
    crossover."""

    def __init__(
        self,
        distribution_index: float | None = None,
        crossover_probability: float | None = None,
        variable_probability: float | None = None,
    ):
        """
        Initialise the argument class.
        :param distribution_index: The distribution index for crossover (this is the
        eta_c in the paper). This directly control the spread of children. If a large
        value is selected, the resulting children will have a higher probability of
        being close to their parents; a small value generates distant offsprings.
        :param crossover_probability: The probability that the parents participate in
        the crossover. If 1.0, the parents always participate in the crossover. If the
        probability is lower, then the children are the exact.
        :param variable_probability: The probability that a variable belonging to both
        parents is used in the crossover. The NSGA2 paper uses 0.5, meaning that  each
        variable in a solution has a 50% chance of changing its value.
        """

class ExportHistory:
    """Configure the export of the algorithm history."""

    generation_step: int
    """Algorithm data will be exported every 'generation_step' generations. """
    destination: Path
    """The path where the serialized results are exported. """

    def __init__(self, generation_step: int, destination: Path):
        """
        Initialise the export history configuration. This returns an error if the
        destination folder does not exist.
        :param generation_step: Export the algorithm data each time the generation
        counter in a genetic algorithm increases by the provided step.
        :param destination: serialise the algorithm history and export the results to a
        JSON file in the given folder.
        """
        ...

class NSGA2Arg:
    """Customise the NSGA2 options."""

    number_of_individuals: int
    """The number of individuals to use in the population."""
    stopping_condition: StoppingCondition
    """The condition to use when to terminate the algorithm."""
    crossover_operator_options: SimulatedBinaryCrossoverArgs | None = None
    """The options of the Simulated Binary Crossover (SBX) operator. """
    mutation_operator_options: PolynomialMutationArgs | None = None
    """The options to Polynomial Mutation (PM) operator."""
    resume_from_file: Path | None = None
    """The path to the JSON file to use to initialise the initial population."""
    parallel: bool | None = True
    """Whether the objective and constraint evaluation should run using threads."""
    export_history: ExportHistory | None = None
    """The options to configure the individual's history export."""
    seed: int | None = None
    """The seed used in the random number generator (RNG)."""

    def __init__(
        self,
        number_of_individuals: int,
        stopping_condition: StoppingCondition,
        crossover_operator_options: SimulatedBinaryCrossoverArgs | None = None,
        mutation_operator_options: PolynomialMutationArgs | None = None,
        resume_from_file: Path | None = None,
        parallel: bool | None = True,
        export_history: ExportHistory | None = None,
        seed: int | None = None,
    ) -> None:
        """
        Input arguments for the NSGA2 algorithm.
        :param number_of_individuals: The number of individuals to use in the
        population.  This must be a multiple of `2`.
        :param stopping_condition: The condition to use when to terminate the algorithm.
        :param crossover_operator_options: The options of the Simulated Binary Crossover
        (SBX) operator. This operator is used to generate new children by recombining
        the variables of parent solutions.
        :param mutation_operator_options:  The options to Polynomial Mutation (PM)
        operator used to mutate the variables of an individual. This defaults to a
        distribution index or index parameter of `20` and variable
        probability equal `1` divided by the number of real variables in the problem
        (i.e., each variable will have the
        same probability of being mutated).
        :param resume_from_file: Instead of initialising the population with random
        variables, see the initial population with the variable values from a JSON files
        exported with this tool. This option lets you restart the evolution
        from a previous generation; you can use any history file (exported when the
        field `export_history`) or the file exported when the stopping condition was
        reached.
        :param parallel: Whether the objective and constraint evaluation should run
        using threads. Defaults to `true` to run the pywr models in parallel.
        :param export_history: The options to configure the individual's history export.
        When provided, the algorithm will save objectives, constraints and solutions to
        a file each time the generation increases by a given step.
        This is useful to track convergence and inspect an algorithm evolution.
        :param seed: The seed used in the random number generator (RNG). You can specify
        a seed in case you want to try to reproduce results. NSGA2 is a stochastic
        algorithm that relies on a RNG at different steps (when population is initially
        generated, during selection, crossover and mutation) and, as such, may lead to
        slightly different solutions. The seed is randomly picked if this is `None`.
        """

class NSGA3Arg:
    """Customise the NSGA2 options."""

    number_of_individuals: int
    """The number of individuals to use in the population."""
    number_of_partitions: int | TwoLayerPartitions
    """The number of partitions to use to generate the reference points."""
    stopping_condition: StoppingCondition
    """The condition to use when to terminate the algorithm."""
    crossover_operator_options: SimulatedBinaryCrossoverArgs | None = None
    """The options of the Simulated Binary Crossover (SBX) operator. """
    mutation_operator_options: PolynomialMutationArgs | None = None
    """The options to Polynomial Mutation (PM) operator."""
    resume_from_file: Path | None = None
    """The path to the JSON file to use to initialise the initial population."""
    parallel: bool | None = True
    """Whether the objective and constraint evaluation should run using threads."""
    export_history: ExportHistory | None = None
    """The options to configure the individual's history export."""
    seed: int | None = None
    """The seed used in the random number generator (RNG)."""

    def __init__(
        self,
        number_of_individuals: int | None,
        number_of_partitions: int | TwoLayerPartitions,
        stopping_condition: StoppingCondition,
        crossover_operator_options: SimulatedBinaryCrossoverArgs | None = None,
        mutation_operator_options: PolynomialMutationArgs | None = None,
        resume_from_file: Path | None = None,
        parallel: bool | None = True,
        export_history: ExportHistory | None = None,
        seed: int | None = None,
    ) -> None:
        """
        Initialise the class.
        :param number_of_individuals: The number of individuals. When `None`, the number
        of individuals are set equal to the number of reference points. Use an integer
        to set a custom number of individuals (this must be larger than the number of
        reference points generated by setting the `number_of_partitions` argument).`
        :param number_of_partitions: Define the number of partitions to use to generate
        the reference points. You can create: 1 layer or set of points with a constant
        uniform gaps by providing an integer. Or 2 layers of points with each layer
        having a different gap by providing an instance of `TwoLayerPartitions`. Use the
        last approach if you are trying to solve a problem with many objectives (4 or
        more) and want to reduce the number of reference points to use. Using two layers
        allows (1) setting a smaller number of reference points, (2) controlling the
        point density in the inner area and (3) ensure a well-spaced point distribution.
        :param stopping_condition: The condition to use when to terminate the algorithm.
        :param crossover_operator_options: The options of the Simulated Binary Crossover
        (SBX) operator. This operator is used to generate new children by recombining
        the variables of parent solutions.
        :param mutation_operator_options:  The options to Polynomial Mutation (PM)
        operator used to mutate the variables of an individual. This defaults to a
        distribution index or index parameter of `20` and variable
        probability equal `1` divided by the number of real variables in the problem
        (i.e., each variable will have the
        same probability of being mutated).
        :param resume_from_file: Instead of initialising the population with random
        variables, see the initial population with the variable values from a JSON files
        exported with this tool. This option lets you restart the evolution
        from a previous generation; you can use any history file (exported when the
        field `export_history`) or the file exported when the stopping condition was
        reached.
        :param parallel: Whether the objective and constraint evaluation should run
        using threads. Defaults to `true` to run the pywr models in parallel.
        :param export_history: The options to configure the individual's history export.
        When provided, the algorithm will save objectives, constraints and solutions to
        a file each time the generation increases by a given step.
        This is useful to track convergence and inspect an algorithm evolution.
        :param seed: The seed used in the random number generator (RNG). You can specify
        a seed in case you want to try to reproduce results. NSGA2 is a stochastic
        algorithm that relies on a RNG at different steps (when population is initially
        generated, during selection, crossover and mutation) and, as such, may lead to
        slightly different solutions. The seed is randomly picked if this is `None`.
        """
        ...

class Algorithm:
    """Select the algorithm to use to solve the optimisation problem."""

    @classmethod
    def nsga2(cls, options: NSGA2Arg) -> Algorithm:
        """
        Use the Non-dominated Sorting Genetic Algorithm (NSGA2) and customise its
        options. Implemented based on:
          K. Deb, A. Pratap, S. Agarwal and T. Meyarivan, "A fast and elitist
          multi-objective genetic algorithm: NSGA-II," in IEEE Transactions on
          Evolutionary Computation, vol. 6, no. 2, pp. 182-197, April 2002,
          doi: 10.1109/4235.996017.
        :param options: The NSGA2Arg instance.
        :return: The Algorithm instance.
        """

    @classmethod
    def nsga3(cls, options: NSGA3Arg) -> Algorithm:
        """
        Use the Non-dominated Sorting Genetic Algorithm (NSGA3) algorithm and
        customise its options. Implemented based on:
          K. Deb and H. Jain, "An Evolutionary Many-Objective Optimization Algorithm
          Using Reference-Point-Based Non-dominated Sorting Approach, Part I: Solving
          Problems With Box Constraints," in IEEE Transactions on Evolutionary
          Computation, vol. 18, no. 4, pp. 577-601, Aug. 2014,
          doi: 10.1109/TEVC.2013.2281535
        :param options: The NSGA3Arg instance.
        :return: The Algorithm instance.
        """

    @classmethod
    def adaptive_nsga3(cls, options: NSGA3Arg) -> Algorithm:
        """
        Use the NSGA3 algorithm with adaptive reference points. This implements the
        new algorithm from Jain and Deb (2014) to handle problems where not all
        reference points intersect the optimal Pareto front. This helps to reduce
        crowding and enhance the solution quality.
        :param options: The NSGA3Arg instance.
        :return: The Algorithm instance.
        """
