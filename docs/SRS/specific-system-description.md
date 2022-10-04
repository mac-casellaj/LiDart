3D scanning is a versatile technology that is used across many industries, but its uses are often limited by high cost and complexity. LiDart aims to build a low cost, simple to use 3D scanning robot. We plan to do this by using easily available low cost LiDAR sensors and consumer grade webcams alongside inexpensive location markers. A software suite will process data obtainted from the robot and provide a user interface. 

LiDart's end product will be a wheel based mobile robot with all required sensors onboard that can be connected to over WiFi.

----------------------------

# Specific System Description

This section first presents the problem description, which gives a high-level view of the problem to be solved. This is followed by the solution characteristics specification, which presents the assumptions, theories, definitions and finally the instance models.

## Problem Description

LiDart is intended to make accurate [1] 3D scanning available to applications where it is currently cost or complexity prohibitive.

To do this, the end product must be accurate [1], low cost and easy to setup and operate.

allow for accurate [1] 3D scans in a low cost, easy to use package.

### Terminology and Definitions

This subsection provides a list of terms that are used in the subsequent sections and their meaning, with the purpose of reducing ambiguity and making it easier to correctly understand the requirements.

- Landmark
    - easily detectable & distinguishable
    - unique
- Localization
    - process of determining the location of an object within its environment
- AprilTag
- LiDAR
    - short for "Light Detection And Ranging"
    - a type of distance sensor
- Point cloud

### Physical System Description

The physical system that LiDart will operate in can be modeled as the environment to be scanned.

### Goal Statements

- Accurate 3D Scanning
    - Ability to product an accurate pointcloud 

- Low Cost Hardware

- Ease of Use

## Solution Characteristics Specification

The solution that LiDart proposes to the problem specified in section 4.1 is a two wheeled mobile robot that can be remotely driven to take 3D scans of its environment.

The robot will have all required sensors onboard, these sensors will include:
- One or more cameras
- A sensor capable of measuring the distance to objects in a given plane relative to the sensor's orientation and position 
- Two sensors capable of measuring wheel rotations, one for each of the two wheels 

- Accurate Localization

### Assumptions

This section simplifies the original problem and helps in developing the theoretical modelby filling in the missing  information for the physical system. The numbers given in the square brackets refer to the theoretical model [T], general definition [GD], data definition[DD], instance model [IM], or likely change [LC], in which the respective assumption is used.

The environment that LiDart is expected to operate within has the following properties:

- Not exposed to the elements (eg. Rain)
- Floor should be flat and suitable for driving on with plastic wheels (dry, hard, not slippery. eg. tile, hardwood, concrete)
- Prepopulated with non-moving landmarks 
- The object to be scanned will not move
- No translucent or reflective objects

- The object to be scanned ??? height ???

- Robot & desktop will be able to communicate over WiFi

 This physical system has the following properties:

- The environment is illuminated such that standard (non low light) cameras can clearly view the surrounding
- The object to be scanned is stationary and will remain stationary for the duration of the 3D scanning process

The LiDart robot itself will have the following properties:
- The ability to move using a wheel based drive train
- The ability to capture images of the scene using 

### Theoretical Models

-- Localization



-- 3D Scanning (Point cloud generation)

### General Definitions

### Data Definitions

### Data Types

### Instance Models

### Input Data Constraints

### Properties of a Correct Solution